//! l96a1-proxy — ProxyScrape fetch + rotacao + health + roteamento hybrid.

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Entrada unica de proxy com score de saude (EMA de latencia).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEntry {
    pub url: String,
    pub protocol: String,
    pub latency_ema_ms: f64,
    pub success: u64,
    pub fails: u64,
    pub cooldown_until: Option<std::time::SystemTime>,
}

impl ProxyEntry {
    pub fn new(url: String) -> Self {
        let protocol = if url.starts_with("socks5") {
            "socks5"
        } else if url.starts_with("socks4") {
            "socks4"
        } else {
            "http"
        }
        .to_string();
        Self { url, protocol, latency_ema_ms: 9999.0, success: 0, fails: 0, cooldown_until: None }
    }

    pub fn healthy(&self) -> bool {
        if self.fails >= 8 {
            return false;
        }
        if let Some(until) = self.cooldown_until {
            if std::time::SystemTime::now() < until {
                return false;
            }
        }
        true
    }

    /// Proxy podre: remover da rotação (chamado pelo prune periódico).
    /// Critério conservador: muitos fails com zero sucesso, ou fails alto
    /// com taxa de sucesso ruim. Flaky (passou >=1) nunca é removido aqui.
    pub fn should_remove(&self) -> bool {
        (self.fails >= 8 && self.success == 0)
            || (self.fails >= 12 && self.success * 4 < self.fails as u64)
    }

    pub fn mark_success(&mut self, latency_ms: f64) {
        self.success += 1;
        self.latency_ema_ms = 0.7 * self.latency_ema_ms.min(9999.0) + 0.3 * latency_ms;
        if self.latency_ema_ms > 9000.0 {
            self.latency_ema_ms = latency_ms;
        }
    }

    pub fn mark_failure(&mut self) {
        self.fails += 1;
        // Cooldown exponencial por falha de rede/timeout (proxy ruim sai da
        // rotação e volta sozinho): 60s → 2min → 4min … teto 15min.
        // Morte permanente só via should_remove() (prune explícito).
        if self.fails >= 3 {
            let shift = (self.fails - 3).min(4) as u32;
            let secs = 60u64.saturating_mul(1 << shift).min(900);
            self.cooldown_until =
                Some(std::time::SystemTime::now() + Duration::from_secs(secs));
        }
    }

    /// Penalidade de rate-limit: além do cooldown por par (Rotator.limited),
    /// empurra o EMA para o proxy ser preterido na distribuição.
    pub fn mark_rate_limited(&mut self) {
        self.latency_ema_ms = (self.latency_ema_ms + 5000.0).min(9999.0);
    }
}

/// Rotacionador: round-robin + sticky por dominio + weighted por latencia.
/// Rate-limit e por par (proxy, plataforma): um proxy limitado no Instagram
/// continua valido para GitHub. Dead (falha de rede) e global.
pub struct Rotator {
    entries: Mutex<Vec<ProxyEntry>>,
    idx: AtomicUsize,
    sticky: Mutex<HashMap<String, usize>>,
    limited: Mutex<HashMap<(String, String), Instant>>,
}

impl Rotator {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            entries: Mutex::new(urls.into_iter().map(ProxyEntry::new).collect()),
            idx: AtomicUsize::new(0),
            sticky: Mutex::new(HashMap::new()),
            limited: Mutex::new(HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }

    pub fn alive_count(&self) -> usize {
        self.entries.lock().unwrap().iter().filter(|e| e.healthy()).count()
    }

    fn prune_limited(&self) {
        let now = Instant::now();
        self.limited.lock().unwrap().retain(|_, until| *until > now);
    }

    /// Marca (proxy, plataforma) como rate-limited por `secs`.
    /// Nao afeta outras plataformas.
    pub fn mark_limited(&self, proxy_url: &str, platform: &str, secs: u64) {
        self.prune_limited();
        self.limited.lock().unwrap().insert(
            (proxy_url.to_string(), platform.to_string()),
            Instant::now() + Duration::from_secs(secs.max(5)),
        );
    }

    fn is_limited(&self, proxy_url: &str, platform: &str) -> bool {
        match self
            .limited
            .lock()
            .unwrap()
            .get(&(proxy_url.to_string(), platform.to_string()))
        {
            Some(until) => *until > Instant::now(),
            None => false,
        }
    }

    /// Escolhe proxy valido para a plataforma: saudavel (global) e sem
    /// cooldown NESTA plataforma. `sticky` fixa um proxy por plataforma.
    /// Distribuição round-robin entre saudáveis (não "sempre o menor EMA",
    /// que concentrava o bulk inteiro num IP só e estourava o shared-bucket).
    /// O EMA segue como sinal de saúde (report/health-check), não como
    /// seletor exclusivo.
    pub fn for_platform(&self, platform: &str, sticky: bool) -> Option<String> {
        self.prune_limited();
        let entries = self.entries.lock().unwrap();
        if entries.is_empty() {
            return None;
        }
        if sticky {
            let sticky_map = self.sticky.lock().unwrap();
            if let Some(&i) = sticky_map.get(platform) {
                if i < entries.len()
                    && entries[i].healthy()
                    && !self.is_limited(&entries[i].url, platform)
                {
                    return Some(entries[i].url.clone());
                }
            }
            drop(sticky_map);
        }
        // Candidatos saudáveis e sem cooldown nesta plataforma.
        let mut cands: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.healthy() && !self.is_limited(&e.url, platform))
            .map(|(i, _)| i)
            .collect();
        if cands.is_empty() {
            // tudo limitado/morto: retorna None para a engine cair em
            // direto ou pular (nunca queima request à toa).
            return None;
        }
        // Ordena por EMA mas escolhe por round-robin entre os 8 melhores:
        // espalha egress (~1-2 reqs/IP) sem ignorar saúde.
        cands.sort_by(|&a, &b| {
            entries[a]
                .latency_ema_ms
                .partial_cmp(&entries[b].latency_ema_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let k = cands.len().min(8);
        let pick = cands[self.idx.fetch_add(1, Ordering::Relaxed) % k];
        let url = entries[pick].url.clone();
        drop(entries);
        if sticky {
            // Refixa o sticky no escolhido (índice pode ter mudado).
            if let Some(pos) = self.snapshot_index(&url) {
                self.sticky.lock().unwrap().insert(platform.to_string(), pos);
            }
        }
        Some(url)
    }

    fn snapshot_index(&self, url: &str) -> Option<usize> {
        // Chamado sem o lock de entries (evita deadlock); custo O(n) ok.
        self.entries.lock().unwrap().iter().position(|e| e.url == url)
    }

    pub fn report(&self, url: &str, ok: bool, latency_ms: f64) {
        if let Some(e) = self.entries.lock().unwrap().iter_mut().find(|e| e.url == url) {
            if ok {
                e.mark_success(latency_ms);
            } else {
                e.mark_failure();
            }
        }
    }

    /// Penalidade de 429 num proxy (chamado junto com mark_limited):
    /// o cooldown por par já tira ele da rotação da plataforma; o EMA
    /// maior faz a distribuição preferir outros IPs ao voltar.
    pub fn report_rate_limited(&self, url: &str) {
        if let Some(e) = self.entries.lock().unwrap().iter_mut().find(|e| e.url == url) {
            e.mark_rate_limited();
        }
    }

    /// Remove proxies podres do pool vivo (prune). Limpa sticky (índices mudam).
    /// Retorna quantos saíram.
    pub fn remove(&self, urls: &[String]) -> usize {
        let mut entries = self.entries.lock().unwrap();
        let before = entries.len();
        entries.retain(|e| !urls.iter().any(|u| u == &e.url));
        self.sticky.lock().unwrap().clear();
        before - entries.len()
    }

    /// Prune automático: remove podres (should_remove) e retorna as URLs.
    /// Chamado pelo worker periódico da API; nunca remove flaky com sucesso.
    pub fn prune_dead(&self) -> Vec<String> {
        let dead: Vec<String> = self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.should_remove())
            .map(|e| e.url.clone())
            .collect();
        if !dead.is_empty() {
            self.remove(&dead);
        }
        dead
    }

    /// Snapshot p/ prune/UI sem expor o lock.
    pub fn snapshot(&self) -> Vec<ProxyEntry> {
        self.entries.lock().unwrap().clone()
    }
}

/// Roteador hybrid: force_proxy x force_direct + fallback_to_direct.
/// Uma-por-uma: cada key em `force_direct` precisa de prova em `research/`
/// (direto decide + proxy cego). Sem prova, remover (vai para pool).
#[derive(Clone)]
pub struct HybridRouter {
    pub force_proxy: Vec<&'static str>,
    pub force_direct: Vec<&'static str>,
    pub fallback_to_direct: bool,
    /// Retry reverso: direct Error -> tenta proxy 1x (aprovado).
    /// Não se aplica a `force_direct` intencional (chesscom/lichess/kick).
    pub fallback_to_proxy: bool,
}

impl Default for HybridRouter {
    fn default() -> Self {
        Self {
            // WAF pesado com afinidade de sessão: proxy primeiro, com
            // fallback p/ direto no retry de Error (live.rs).
            // TikTok SAIU da lista (research/tiktok.md 2026-10): oembed e page
            // vivem em buckets independentes e o teto via proxy é >24/IP —
            // sticky concentrava o bulk inteiro num IP só (o 1/94 de rate).
            // needs_proxy=true já garante pool; rotação = ~1-2 reqs/IP.
            force_proxy: vec!["instagram", "x", "snapchat", "facebook"],
            // Uma-por-uma (research/ como prova):
            // MANTER direto só com prova "direto decide + proxy cego":
            // - chesscom: proxy 403 CF ambos, direto 200+username/404 (chess_com.md)
            // - lichess: proxy 000 timeout ambos, direto 200/404 (lichess.md)
            // - kick: sem WAF, direto funciona, manter direct (kick.md)
            // REMOVIDOS (iam direto erroneamente, agora pool/direct + retry reverso):
            // - minecraft: validado via proxy 200+id/404 (minecraft.md)
            // - github/gitlab: validados via proxy, direto não testado (github.md/gitlab.md)
            // - youtube/roblox/steam/soundcloud/myspace/gunslol/telegram/
            //   pinterest/twitch/threads/bluesky/spotify: research valida via
            //   proxy; direto sem prova ou cego — vão para pool.
            // - reddit/imgur/medium/tumblr: ambas rotas cegas hoje (reddit.md,
            //   imgur.md, medium.md, tumblr.md) — pool/direct com Unknown
            //   honesto, nunca Available chutado; sem force_direct.
            // - credly/shelf/mercadolivre: sem research (comentário audit-2026-10
            //   órfão) — fora até ter prova.
            force_direct: vec![
                "chesscom", "lichess", "kick",
            ],
            fallback_to_direct: true,
            fallback_to_proxy: true,
        }
    }
}

impl HybridRouter {
    /// Chave base p/ as listas: `mf:github` e `github` roteiam igual.
    fn base_key(platform_key: &str) -> &str {
        platform_key.rsplit(':').next().unwrap_or(platform_key)
    }

    /// `true` se a plataforma é direta intencional (não sofre retry reverso).
    pub fn is_force_direct(&self, platform_key: &str) -> bool {
        self.force_direct.contains(&Self::base_key(platform_key))
    }

    /// None = rota direta. Quando o pool esvazia (tudo dead/limited),
    /// cai em direto se `fallback_to_direct` (default true).
    /// `needs_proxy=true` vence `force_direct`: plataforma que precisa de
    /// proxy nunca vai direto se há proxy vivo (uma-por-uma).
    pub fn use_proxy(
        &self,
        platform_key: &str,
        sticky: bool,
        needs_proxy: bool,
        rotator: &Rotator,
    ) -> Option<String> {
        let base = Self::base_key(platform_key);
        if needs_proxy {
            let s = sticky || self.force_proxy.contains(&base);
            // Se pool vazio retorna None; o chamador (probe_one) deve tratar
            // como "sem rota" e NÃO como direct silencioso.
            return rotator.for_platform(platform_key, s);
        }
        if self.force_direct.contains(&base) {
            return None;
        }
        let sticky = sticky || self.force_proxy.contains(&base);
        rotator.for_platform(platform_key, sticky)
    }

    /// Classificacao legivel da rota de uma plataforma (p/ HUD e doctor).
    pub fn route_label(
        &self,
        platform_key: &str,
        profile: &l96a1_core::PlatformProfile,
    ) -> &'static str {
        if self.force_direct.contains(&Self::base_key(platform_key)) {
            "direct"
        } else if profile.sticky_proxy || self.force_proxy.contains(&Self::base_key(platform_key)) {
            "sticky-proxy"
        } else if profile.needs_proxy {
            "proxy-req"
        } else {
            "pool/direct"
        }
    }
}

/// Baixa lista free do ProxyScrape v4 (sem auth).
pub async fn fetch_proxyscrape(
    client: &reqwest::Client,
    protocols: &str,
    limit: usize,
) -> AnyhowResult<Vec<String>> {
    let url = format!(
        "https://api.proxyscrape.com/v4/free-proxy-list/get?request=getproxies&protocol={protocols}&timeout=10000&limit={limit}"
    );
    let text = client.get(&url).send().await?.text().await?;
    Ok(text.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect())
}

type AnyhowResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Health-check paralelo via `http://gstatic.com/generate_204` (espera 204).
pub async fn health_check(entries: &mut [ProxyEntry], concurrency: usize) {
    health_check_targets(entries, &["http://gstatic.com/generate_204"], concurrency).await;
}

/// Resultado por proxy contra alvos reais: vivo se passou em >=1 alvo.
/// `ok_targets` conta em quantos alvos passou (p/ UI); latência = melhor.
pub struct TargetVerdict {
    pub ok: bool,
    pub ok_targets: usize,
    pub targets: usize,
    pub latency_ms: Option<f64>,
}

/// Health-check real: cada proxy contra cada alvo (2xx = ok).
/// gstatic valida conectividade; discord.com/github.com validam utilidade
/// para o username checker (um proxy pode passar num e falhar noutro).
pub async fn health_check_targets(
    entries: &mut [ProxyEntry],
    targets: &[&str],
    concurrency: usize,
) -> Vec<(String, TargetVerdict)> {
    let jobs: Vec<(String, String)> = entries
        .iter()
        .flat_map(|e| targets.iter().map(|t| (e.url.clone(), t.to_string())))
        .collect();
    let per: Vec<(String, bool, Option<f64>)> = stream::iter(jobs)
        .map(|(url, target)| async move {
            let c = l96a1_core::client_for(Duration::from_secs(6), Some(&url));
            let t = Instant::now();
            let ok = c
                .get(&target)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);
            let lat = ok.then(|| t.elapsed().as_secs_f64() * 1000.0);
            (url, ok, lat)
        })
        .buffer_unordered(concurrency.max(1))
        .collect()
        .await;
    // Agrega por proxy: melhor latência + conta alvos ok.
    let mut agg: HashMap<String, (usize, Option<f64>)> = HashMap::new();
    for (url, ok, lat) in per {
        let e = agg.entry(url).or_insert((0, None));
        if ok {
            e.0 += 1;
            e.1 = Some(e.1.map_or(lat.unwrap_or(9999.0), |best| best.min(lat.unwrap_or(9999.0))));
        }
    }
    let mut out = Vec::new();
    for e in entries.iter_mut() {
        let (n, lat) = agg.remove(&e.url).unwrap_or((0, None));
        let verdict = TargetVerdict {
            ok: n > 0,
            ok_targets: n,
            targets: targets.len(),
            latency_ms: lat,
        };
        // UMA marca por proxy (conservador): ok = success, fail-all = 1 failure.
        // Flaky (passou >=1) nunca morre por causa do health-check.
        if verdict.ok {
            e.mark_success(lat.unwrap_or(9999.0));
        } else {
            e.mark_failure();
        }
        out.push((e.url.clone(), verdict));
    }
    out
}

/// Carrega `proxies.txt` (ignora `#` e vazias, normaliza sem esquema p/ http://).
pub fn load_file(path: &str) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            if l.contains("://") {
                l.to_string()
            } else {
                format!("http://{l}")
            }
        })
        .collect()
}

pub fn _arc<T>(t: T) -> Arc<T> {
    Arc::new(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_platform_cooldown_keeps_proxy_alive_for_other_sites() {
        let rot = Rotator::new(vec!["http://1.1.1.1:8080".into()]);
        assert_eq!(rot.len(), 1);
        assert!(!rot.is_empty());
        assert_eq!(rot.alive_count(), 1);

        // Limitado no Instagram nao afeta o GitHub ou TikTok
        rot.mark_limited("http://1.1.1.1:8080", "instagram", 60);
        assert_eq!(rot.for_platform("instagram", true), None);
        assert_eq!(
            rot.for_platform("tiktok", true),
            Some("http://1.1.1.1:8080".into())
        );
    }

    #[test]
    fn remove_drops_entries_and_clears_sticky() {
        let rot = Rotator::new(vec!["http://1.1.1.1:8080".into(), "http://2.2.2.2:8080".into()]);
        assert_eq!(rot.remove(&["http://1.1.1.1:8080".to_string()]), 1);
        assert_eq!(rot.len(), 1);
        assert_eq!(rot.remove(&["http://9.9.9.9:9999".to_string()]), 0);
    }

    #[test]
    fn hybrid_router_respects_force_direct_and_sticky() {
        let rot = Rotator::new(vec!["http://2.2.2.2:8080".into()]);
        let router = HybridRouter::default();
        // Diretas intencionais com prova (chess_com.md/lichess.md/kick.md).
        assert_eq!(router.use_proxy("lichess", false, false, &rot), None);
        assert_eq!(router.use_proxy("mf:lichess", false, false, &rot), None);
        assert_eq!(
            router.use_proxy("instagram", true, true, &rot),
            Some("http://2.2.2.2:8080".into())
        );
    }

    #[test]
    fn tiktok_rotates_via_pool_not_sticky() {
        // REGRESSÃO (bulk 1/94): tiktok era force_proxy (sticky) e o bulk
        // inteiro caía num IP só. needs_proxy=true garante pool;
        // rotação espalha ~1-2 reqs/IP (research/tiktok.md).
        let rot = Rotator::new(vec!["http://9.9.9.9:8080".into()]);
        let router = HybridRouter::default();
        assert!(!router.force_proxy.contains(&"tiktok"));
        let profile = l96a1_core::PlatformProfile { max_concurrent: 12, needs_proxy: true, sticky_proxy: false };
        assert!(router.use_proxy("tiktok", false, true, &rot).is_some());
        assert_eq!(router.route_label("tiktok", &profile), "proxy-req");
    }

    #[test]
    fn discord_routes_via_pool_not_direct() {
        // Reversão do commit direct-first: shared-bucket por IP exige
        // espalhar egress; nativa e mf: roteiam igual (base_key).
        let rot = Rotator::new(vec!["http://9.9.9.9:8080".into()]);
        let router = HybridRouter::default();
        let profile = l96a1_core::PlatformProfile::default();
        assert!(router.use_proxy("discord", false, false, &rot).is_some());
        assert!(router.use_proxy("mf:discord", false, false, &rot).is_some());
        assert_eq!(router.route_label("discord", &profile), "pool/direct");
        assert_eq!(router.route_label("mf:discord", &profile), "pool/direct");
        // controle: lichess segue direct (prova em research/lichess.md)
        assert!(router.use_proxy("lichess", false, false, &rot).is_none());
    }

    #[test]
    fn minecraft_routes_via_pool_not_direct() {
        // Uma-por-uma Fase 1: research/minecraft.md valida via proxy;
        // `minecraft` saiu de `force_direct` (regressão corrigida).
        let rot = Rotator::new(vec!["http://9.9.9.9:8080".into()]);
        let router = HybridRouter::default();
        assert!(!router.is_force_direct("minecraft"));
        assert!(!router.is_force_direct("mf:minecraft"));
        assert!(router.use_proxy("minecraft", false, false, &rot).is_some());
        assert!(router.use_proxy("mf:minecraft", false, false, &rot).is_some());
    }

    #[test]
    fn needs_proxy_beats_force_direct_when_pool_alive() {
        // Base comum: `needs_proxy=true` vence `force_direct` se há proxy vivo.
        let rot = Rotator::new(vec!["http://9.9.9.9:8080".into()]);
        let router = HybridRouter::default();
        assert!(router.is_force_direct("lichess"));
        assert!(router.use_proxy("lichess", false, true, &rot).is_some());
    }

    #[test]
    fn one_by_one_pool_list_routes_via_pool() {
        // Uma-por-uma Fase 3/5: research valida via pool (ou ambas cegas);
        // nenhuma dessas pode ser `force_direct`.
        let rot = Rotator::new(vec!["http://9.9.9.9:8080".into()]);
        let router = HybridRouter::default();
        for k in [
            "github", "mf:github", "gitlab", "youtube", "reddit", "imgur", "roblox",
            "minecraft", "mf:minecraft", "spotify", "steam", "soundcloud", "medium",
            "myspace", "gunslol", "tumblr", "telegram", "pinterest", "twitch",
            "threads", "bluesky", "credly", "shelf", "mercadolivre",
        ] {
            assert!(!router.is_force_direct(k), "{k}");
            assert!(router.use_proxy(k, false, false, &rot).is_some(), "{k}");
        }
        // Fase 4: só diretas com prova ficam.
        for k in ["chesscom", "mf:chesscom", "lichess", "kick"] {
            assert!(router.is_force_direct(k), "{k}");
            assert!(router.use_proxy(k, false, false, &rot).is_none(), "{k}");
        }
    }

    #[test]
    fn distributes_across_proxies_not_always_best_ema() {
        // Com 4 proxies saudáveis, 40 escolhas devem cobrir os 4 (round-robin
        // entre top-K) — antes era sempre o menor EMA (1 IP levava o bulk).
        let rot = Rotator::new(vec![
            "http://10.0.0.1:8080".into(),
            "http://10.0.0.2:8080".into(),
            "http://10.0.0.3:8080".into(),
            "http://10.0.0.4:8080".into(),
        ]);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..40 {
            seen.insert(rot.for_platform("discord", false).unwrap());
        }
        assert_eq!(seen.len(), 4);
    }

    #[test]
    fn limited_proxy_leaves_rotation_until_cooldown() {
        let rot = Rotator::new(vec!["http://10.0.1.1:8080".into(), "http://10.0.1.2:8080".into()]);
        rot.mark_limited("http://10.0.1.1:8080", "discord", 600);
        // 10 escolhas seguidas: só o .2 aparece (o .1 está de cooldown).
        for _ in 0..10 {
            assert_eq!(
                rot.for_platform("discord", false),
                Some("http://10.0.1.2:8080".into())
            );
        }
        // Outra plataforma não é afetada (cooldown é por par).
        assert!(rot.for_platform("github", false).is_some());
    }

    #[test]
    fn prune_dead_removes_only_rotten() {
        let rot = Rotator::new(vec!["http://10.0.2.1:8080".into(), "http://10.0.2.2:8080".into()]);
        // .1: 8 fails sem sucesso = podre; .2: flaky com sucesso = mantém.
        for _ in 0..8 {
            rot.report("http://10.0.2.1:8080", false, 0.0);
        }
        rot.report("http://10.0.2.2:8080", false, 0.0);
        rot.report("http://10.0.2.2:8080", true, 200.0);
        let dead = rot.prune_dead();
        assert_eq!(dead, vec!["http://10.0.2.1:8080".to_string()]);
        assert_eq!(rot.len(), 1);
    }
}
