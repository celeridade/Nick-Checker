//! Engine web (lógica portada do antigo CLI `ui/live.rs`, removido na Fase 5):
//! semáforo global + por plataforma, HybridRouter, retry, AIMD-lite,
//! throttle batch/delay — emite eventos para SSE / jobs.

use crate::models::SearchResult;
use l96a1_core::{CheckCtx, CheckOutcome, CheckResult, Platform};
use l96a1_proxy::{HybridRouter, Rotator};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

/// Circuit breaker por plataforma contra 429 de escopo compartilhado/global.
///
/// Quando um check volta `RateLimited` com `retry_after` longo, o bucket é do
/// IP (ex.: Discord `scope=shared 985s`). Com pool de ~100 proxies, 1 evento
/// isolado é só 1 IP ruim — o cooldown por par (Rotator.limited) já tira ele
/// da rotação. O breaker global só arma por QUORUM: >=3 eventos shared/global
/// no mesmo bucket nos últimos 120s. Sem quorum, os outros 99 IPs seguem.
/// Vale para todos os modos (single/bulk/job/sniper) pois todos passam por
/// `probe_one`. Backoff curto (<60s) ou escopo `user` não arma (ruído local).
static PLATFORM_BACKOFF: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, u64>>> =
    std::sync::OnceLock::new();

/// Eventos shared/global recentes: (bucket, instante, until_epoch).
/// Base do quorum — 1 proxy rateado não para o mundo.
static SHARED_EVENTS: std::sync::OnceLock<std::sync::Mutex<Vec<(String, Instant, u64)>>> =
    std::sync::OnceLock::new();

/// Quorum para armar o breaker global + janela de observação.
const BACKOFF_QUORUM: usize = 3;
const BACKOFF_WINDOW_SECS: u64 = 120;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Chave do bucket: sufixo após `:` — `mf:discord` e `discord` dividem o
/// mesmo host e portanto o mesmo bucket.
pub fn backoff_bucket(platform_key: &str) -> String {
    platform_key.rsplit(':').next().unwrap_or(platform_key).to_string()
}

/// Extrai `(scope, segundos)` de evidence `retry_after scope=<s> <secs>s`.
pub fn parse_retry_after(evidence: &str) -> Option<(String, f64)> {
    let after = evidence.find("retry_after scope=")?;
    let rest = &evidence[after + "retry_after scope=".len()..];
    let mut parts = rest.split_whitespace();
    let scope = parts.next()?.to_string();
    let secs: f64 = parts.next()?.trim_end_matches('s').parse().ok()?;
    Some((scope, secs))
}

const BACKOFF_MIN_SECS: f64 = 60.0;
const BACKOFF_MAX_SECS: u64 = 1800;

/// Segundos restantes de backoff (0 = livre).
pub fn backoff_remaining(bucket: &str) -> u64 {
    let map = PLATFORM_BACKOFF.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let guard = map.lock().unwrap();
    guard.get(bucket).map(|until| until.saturating_sub(now_secs())).unwrap_or(0)
}

/// Arma o breaker a partir de um resultado RateLimited (chamado 1x por probe).
/// Quorum: registra o evento e só trava o bucket global com >=3 eventos
/// shared/global em 120s. Evento isolado = só cooldown do par (proxy, plat).
pub fn note_ratelimited(platform_key: &str, evidence: &str) {
    let Some((scope, secs)) = parse_retry_after(evidence) else { return };
    if (scope == "shared" || scope == "global") && secs >= BACKOFF_MIN_SECS {
        let bucket = backoff_bucket(platform_key);
        let until = now_secs() + (secs as u64).min(BACKOFF_MAX_SECS);
        let events = SHARED_EVENTS.get_or_init(|| std::sync::Mutex::new(Vec::new()));
        let mut ev = events.lock().unwrap();
        let now_i = Instant::now();
        ev.push((bucket.clone(), now_i, until));
        // Prune: fora da janela ou expirados.
        ev.retain(|(_, t, u)| t.elapsed().as_secs() <= BACKOFF_WINDOW_SECS && *u > now_secs());
        let quorum = ev.iter().filter(|(b, _, _)| *b == bucket).count();
        drop(ev);
        if quorum >= BACKOFF_QUORUM {
            let map = PLATFORM_BACKOFF.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
            map.lock().unwrap().insert(bucket, until);
        }
    }
}

/// Limpa eventos/backoff (testes).
#[cfg(test)]
fn reset_backoff_for_tests() {
    if let Some(m) = PLATFORM_BACKOFF.get() {
        m.lock().unwrap().clear();
    }
    if let Some(e) = SHARED_EVENTS.get() {
        e.lock().unwrap().clear();
    }
}

/// Dedup de nicks entre todos os modos de scan: trim, sem @ inicial,
/// vazios fora, case-insensitive preservando a primeira ocorrência.
/// Garante que nenhum scan cheque o mesmo user 2x na mesma passada.
pub fn dedupe_usernames(users: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for u in users {
        let t = u.trim().trim_start_matches('@').to_string();
        if t.is_empty() {
            continue;
        }
        if seen.insert(t.to_lowercase()) {
            out.push(t);
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub timeout_secs: u64,
    pub max_concurrent: usize,
    pub github_token: String,
    pub throttle_batch: usize,
    pub throttle_delay_secs: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            max_concurrent: 100,
            github_token: String::new(),
            throttle_batch: 0,
            throttle_delay_secs: 30.0,
        }
    }
}

/// Extrai `https://host` de `https://host/path...` sem crate `url`.
pub fn origin_of(full: &str) -> String {
    if let Some(rest) = full.strip_prefix("https://") {
        let host = rest.split('/').next().unwrap_or(rest);
        return format!("https://{host}");
    }
    if let Some(rest) = full.strip_prefix("http://") {
        let host = rest.split('/').next().unwrap_or(rest);
        return format!("http://{host}");
    }
    full.to_string()
}

/// Resolve `all,social,mf:github,...` direto no registro unificado
/// (29 nativas + ~1176 geradas; ver `tools/gen_sites.py`).
/// Prefixos `mf:`/`wm:` legados passam direto: viraram keys de verdade.
pub fn resolve_platforms(query: &str) -> Vec<Arc<dyn Platform + 'static>> {
    l96a1_checkers::resolve(query)
}

/// Resultado bruto de 1 check (qualquer `CheckResult`).
/// O sniper precisa de `Available` (nick liberado); o search só emite hits.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Probe {
    pub norm: String,
    pub result: CheckResult,
    pub latency_ms: u64,
    pub evidence: String,
    pub profile_url: String,
    pub platform_key: &'static str,
    pub platform_name: String,
    pub proxy: Option<String>,
    pub attempt: u32,
    pub cached: bool,
}

/// Roda 1 check com retry bidirecional (cooldown por par + fallback).
/// proxy Error -> direto 1x; direto Error -> proxy 1x (exceto force_direct).
/// `None` = nick inválido para a plataforma (sem rede).
pub async fn probe_one(
    plat: Arc<dyn Platform + 'static>,
    raw_user: &str,
    base_client: Arc<reqwest::Client>,
    rotator: Arc<Rotator>,
    router: &HybridRouter,
    discord_tokens: &[String],
    tiktok_sessions: &[String],
    cfg: &EngineConfig,
) -> Option<Probe> {
    let bucket = backoff_bucket(plat.key());
    // Bucket em backoff: pula o HTTP (não queima request no 429).
    if backoff_remaining(&bucket) > 0 {
        let norm = plat.validator().normalize(raw_user)?;
        let name = plat.name().to_string();
        let profile_url = plat.profile_url(&norm);
        let key = plat.key();
        return Some(Probe {
            norm,
            result: CheckResult::RateLimited,
            latency_ms: 0,
            evidence: format!("backoff {bucket} (sem request)"),
            profile_url,
            platform_key: key,
            platform_name: name,
            proxy: None,
            attempt: 0,
            cached: false,
        });
    }
    let norm = plat.validator().normalize(raw_user)?;
    let key = plat.key();
    let name = plat.name().to_string();
    let profile_url = plat.profile_url(&norm);
    let profile = plat.profile();
    let mut attempt = 0u32;
    let mut force_direct_retry = false;
    let mut force_proxy_retry = false;
    let mut last_proxy: Option<String>;
    // assigned on first loop iteration below
    let outcome = loop {
        let proxy = if force_direct_retry {
            force_direct_retry = false;
            None
        } else if force_proxy_retry {
            force_proxy_retry = false;
            // Retry reverso (aprovado): ignora `force_direct` e tenta o pool.
            // `sticky` do perfil é respeitado; `for_platform` pode devolver
            // None se o pool esvaziou entre as tentativas.
            rotator.for_platform(plat.key(), profile.sticky_proxy)
        } else {
            router.use_proxy(
                plat.key(),
                profile.sticky_proxy,
                profile.needs_proxy,
                &rotator,
            )
        };
        // Anti direct-silencioso (uma-por-uma): `needs_proxy` sem proxy
        // vivo/livre NESTA plataforma não queima request direto que será
        // bloqueado — retorna barreira honesta sem HTTP.
        if proxy.is_none() && profile.needs_proxy {
            last_proxy = None;
            break CheckOutcome::new(
                CheckResult::RateLimited,
                0,
                format!("no-proxy-alive {key} (sem request)"),
            );
        }
        last_proxy = proxy.clone();
        let ctx = CheckCtx {
            client: base_client.clone(),
            proxy_url: proxy.clone(),
            discord_tokens: discord_tokens.to_vec(),
            tiktok_sessions: tiktok_sessions.to_vec(),
            github_token: if cfg.github_token.is_empty() {
                None
            } else {
                Some(cfg.github_token.clone())
            },
            timeout: Duration::from_secs(cfg.timeout_secs),
        };
        let o = plat.check(&norm, &ctx).await;
        match o.result {
            CheckResult::Available | CheckResult::Taken | CheckResult::Private | CheckResult::Banned => {
                if let Some(px) = &proxy {
                    rotator.report(px, true, o.latency_ms as f64);
                }
            }
            CheckResult::Error => {
                if let Some(px) = &proxy {
                    rotator.report(px, false, 0.0);
                }
            }
            _ => {}
        }
        if o.result == CheckResult::RateLimited {
            // Cooldown do par (proxy, plataforma) com o retry_after REAL da
            // evidência (não fixo): proxy rateado sai da rotação até liberar
            // e o EMA é penalizado para a distribuição preferir outros IPs.
            // 403 dc-block (datacenter recusado) também tira o IP da rotação
            // do Discord por 5min — retentar no mesmo IP é queimar request.
            if let Some(px) = &proxy {
                let secs = parse_retry_after(&o.evidence)
                    .map(|(_, s)| s.ceil() as u64)
                    .unwrap_or(60)
                    .clamp(5, 1800);
                rotator.mark_limited(px, plat.key(), secs);
                rotator.report_rate_limited(px);
            } else if o.evidence.contains("dc-block-403") {
                // Direto recusado pelo Discord: sem proxy p/ marcar; o retry
                // reverso abaixo tenta o pool antes de desistir.
            }
            // Parede shared/global (bucket do IP): retentar queima request
            // à toa — sai direto e o breaker arma após o loop. Retry só vale
            // p/ transiente ou escopo user.
            let shared_wall = parse_retry_after(&o.evidence)
                .map(|(s, _)| s == "shared" || s == "global")
                .unwrap_or(false);
            if !shared_wall && attempt == 0 {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(900)).await;
                continue;
            }
        }
        if o.result == CheckResult::Error && attempt == 0 {
            attempt += 1;
            if let Some(px) = &proxy {
                rotator.mark_limited(px, plat.key(), 30);
                // `needs_proxy` nunca cai para direto (evita direto sabidamente bloqueado).
                force_direct_retry = !profile.needs_proxy && proxy.is_some() && router.fallback_to_direct;
            } else {
                // Era direto: retry reverso 1x via pool, exceto direta
                // intencional (chesscom/lichess/kick) onde proxy é sabidamente cego.
                force_proxy_retry = router.fallback_to_proxy
                    && !router.is_force_direct(plat.key())
                    && rotator.alive_count() > 0;
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
            continue;
        }
        // Bloco de datacenter (Discord 403 code 40333): IP recusado, não é
        // throttling — tira o IP da rotação do Discord por 5min e retenta 1x
        // noutro egress (rotação faz o resto nos próximos nicks).
        if o.result == CheckResult::Unknown && o.evidence.contains("dc-block-403") && attempt == 0 {
            if let Some(px) = &proxy {
                rotator.mark_limited(px, plat.key(), 300);
                rotator.report_rate_limited(px);
            }
            attempt += 1;
            tokio::time::sleep(Duration::from_millis(300)).await;
            continue;
        }
        break o;
    };
    if outcome.result == CheckResult::RateLimited {
        note_ratelimited(key, &outcome.evidence);
    }

    Some(Probe {
        norm,
        result: outcome.result,
        latency_ms: outcome.latency_ms,
        evidence: outcome.evidence,
        profile_url,
        platform_key: key,
        platform_name: name,
        proxy: last_proxy,
        attempt,
        cached: false,
    })
}

/// Classifica 1 probe: hit (taken/private/banned) vira SearchResult;
/// Available vira FreeNick (vai para resultados/download); rate/error/unknown
/// viram Warn (RateBox + tabela, Fase 4) em vez de sumir no `checked`.
pub enum ProbeClass {
    Hit(SearchResult),
    Free(crate::models::FreeNick),
    Warn(crate::models::WarnInfo),
    Ignore,
}

pub fn classify_probe(
    probe: Probe,
    plat_group: &str,
    source_query: &str,
    display_order: usize,
) -> ProbeClass {
    match probe.result {
        CheckResult::Taken | CheckResult::Private | CheckResult::Banned => {
            let profile_url = probe.profile_url.clone();
            // Avatar determinístico sem API extra (research fingerprint):
            // - minecraft / namemc: cabeça 2D via mc-heads por nick
            //   (com UUID real vira /avatar/<uuid>; sem UUID, /avatar/<nick>
            //   resolve igual). NameMC 2D ideal
            //   s.namemc.com/2d/skin/face.png?id=<hash>&scale=8 precisa do
            //   hash da página — mc-heads é o fallback sem scrape.
            // - discord: sem hash do CDN não dá p/ montar
            //   cdn.discordapp.com/avatars/<id>/<hash>.png; o perfil
            //   https://discord.com/users/<id> exibe foto + banner (sólido
            //   mesmo sem banner) e o frontend usa isso na embed.
            let key = probe.platform_key;
            let pfp: Option<String> = match key {
                "minecraft" | "namemc" => {
                    Some(format!("https://mc-heads.net/avatar/{}", probe.norm))
                }
                _ => None,
            };
            ProbeClass::Hit(SearchResult {
                platform: probe.platform_name.clone(),
                platformUrl: origin_of(&profile_url),
                username: probe.norm.clone(),
        displayName: None,
        pfpUrl: pfp,
        bannerUrl: None,
        userId: None,
        bio: None,
        followers: None,
        following: None,
        posts: None,
        likes: None,
        isVerified: None,
        emails: Vec::new(),
        website: None,
        createdAt: None,
        lastActive: None,
        location: None,
        isPrivate: probe.result == CheckResult::Private,
        locked: true,
        extras: HashMap::new(),
        profileUrl: profile_url,
        sourceQuery: source_query.to_string(),
        duration: format!("{}ms", probe.latency_ms),
        category: plat_group.to_string(),
        hasChecker: true,
        displayOrder: display_order,
        cached: probe.cached,
            })
        }
        CheckResult::Available => ProbeClass::Free(crate::models::FreeNick {
            nick: probe.norm.clone(),
            platform: probe.platform_name.clone(),
            profileUrl: probe.profile_url.clone(),
        }),
        CheckResult::RateLimited | CheckResult::Error | CheckResult::Unknown => {
            let reason = match probe.result {
                CheckResult::RateLimited => "rate",
                CheckResult::Error => "error",
                _ => "unknown",
            };
            ProbeClass::Warn(crate::models::WarnInfo {
                nick: source_query.to_string(),
                platform: probe.platform_name.clone(),
                key: probe.platform_key.to_string(),
                reason: reason.to_string(),
                evidence: probe.evidence.clone(),
                latency_ms: probe.latency_ms,
                proxy: l96a1_core::mask_proxy(probe.proxy.as_deref()),
                attempt: probe.attempt,
            })
        }
    }
}

/// Roda `users x platforms` com concorrência por perfil.
/// `on_hit` / `on_free` / `on_warn` / `on_checked` são chamados conforme
/// cada check termina (ordem de chegada, igual ao stream CLI).
/// Retorna (hits, frees, checked, warns, rates, errors, unknowns).
pub async fn run_scan<F1, F2, F3, F4>(
    users: Vec<String>,
    platforms: Vec<Arc<dyn Platform + 'static>>,
    rotator: Arc<Rotator>,
    discord_tokens: Vec<String>,
    tiktok_sessions: Vec<String>,
    cfg: EngineConfig,
    mut on_hit: F1,
    mut on_free: F2,
    mut on_warn: F4,
    mut on_checked: F3,
    cancel: Arc<AtomicBool>,
    cache: Arc<crate::checked::CheckedCache>,
) -> (usize, usize, usize, usize, usize, usize, usize)
where
    F1: FnMut(SearchResult) + Send + 'static,
    F2: FnMut(crate::models::FreeNick) + Send + 'static,
    F3: FnMut(usize) + Send + 'static,
    F4: FnMut(crate::models::WarnInfo) + Send + 'static,
{
    // Defesa em profundidade: dedupe aqui também (frontend/backend já dedupam).
    let users = dedupe_usernames(users);
    if users.is_empty() || platforms.is_empty() {
        return (0, 0, 0, 0, 0, 0, 0);
    }
    let global_cap = cfg.max_concurrent.clamp(1, 300);
    let global_sem = Arc::new(tokio::sync::Semaphore::new(global_cap));
    let mut plat_sems: HashMap<String, Arc<tokio::sync::Semaphore>> = HashMap::new();
    for p in &platforms {
        let want = p.profile().max_concurrent.clamp(1, global_cap);
        plat_sems
            .entry(p.key().to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Semaphore::new(want)));
    }
    let plat_sems = Arc::new(plat_sems);
    let buckets = Arc::new(tokio::sync::Mutex::new(HashMap::<String, Instant>::new()));
    let sched_cnt = Arc::new(tokio::sync::Mutex::new(HashMap::<String, usize>::new()));
    let router = HybridRouter::default();
    let base_client = Arc::new(l96a1_core::build_client(
        Duration::from_secs(cfg.timeout_secs),
        None,
    ));

    // displayOrder estável por plataforma (índice no resolve).
    let order: HashMap<String, usize> = platforms
        .iter()
        .enumerate()
        .map(|(i, p)| (p.key().to_string(), i))
        .collect();

    let mut jobs: Vec<(String, Arc<dyn Platform + 'static>)> = Vec::new();
    for u in &users {
        for p in &platforms {
            jobs.push((u.clone(), p.clone()));
        }
    }
    let _total_jobs = jobs.len();
    let mut checked = 0usize;
    let mut hits = 0usize;

    // JoinSet em vez de stream::iter().map().buffer_unordered():
    // evita HRTB de `dyn Platform` e garante Future Send + 'static
    // para o tokio::spawn externo das rotas SSE/jobs.
    let mut set = tokio::task::JoinSet::new();
    for (user, plat) in jobs {
        let global_sem = global_sem.clone();
        let plat_sems = plat_sems.clone();
        let buckets = buckets.clone();
        let sched_cnt = sched_cnt.clone();
        let rotator = rotator.clone();
        let base_client = base_client.clone();
        let discord_tokens = discord_tokens.clone();
        let tiktok_sessions = tiktok_sessions.clone();
        let cfg = cfg.clone();
        let router = router.clone();
        let order = order.clone();
        let cancel = cancel.clone();
        let cache = cache.clone();
        set.spawn(async move {
            // Cancelado antes de começar: pula sem contar em checked.
            if cancel.load(Ordering::Relaxed) {
                return None as Option<ProbeClass>;
            }
            // Cache de já-checados (sem rede): nick normalizado + plataforma.
            let norm_opt = plat.validator().normalize(&user);
            if let Some(ref norm) = norm_opt {
                if let Some((cached_result, latency)) = cache.get(norm, plat.key()).await {
                    let profile_url = plat.profile_url(norm);
                    let group = plat.group().to_string();
                    let display_order = *order.get(plat.key()).unwrap_or(&0);
                    let probe = Probe {
                        norm: norm.clone(),
                        result: cached_result,
                        latency_ms: latency,
                        evidence: "cached".to_string(),
                        profile_url,
                        platform_key: plat.key(),
                        platform_name: plat.name().to_string(),
                        proxy: None,
                        attempt: 0,
                        cached: true,
                    };
                    let class = classify_probe(probe, &group, &user, display_order);
                    return Some(class) as Option<ProbeClass>;
                }
            }
            if cfg.throttle_batch > 0 {
                let hit = {
                    let mut m = sched_cnt.lock().await;
                    let n = m.entry(plat.key().to_string()).or_insert(0);
                    *n += 1;
                    *n % cfg.throttle_batch == 0
                };
                if hit {
                    tokio::time::sleep(Duration::from_secs_f64(cfg.throttle_delay_secs)).await;
                }
            }
            let _g = global_sem.clone().acquire_owned().await.ok()?;
            let psem = plat_sems.get(plat.key()).cloned();
            let _p = match psem {
                Some(s) => s.acquire_owned().await.ok()?,
                None => global_sem.clone().acquire_owned().await.ok()?,
            };
            let wait_ms = {
                let mut b = buckets.lock().await;
                let now = Instant::now();
                let min = plat.min_interval_ms();
                let wait = match b.get(plat.key()) {
                    Some(prev) => {
                        let el = now.duration_since(*prev).as_millis() as u64;
                        if el < min { min - el } else { 0 }
                    }
                    None => 0,
                };
                b.insert(plat.key().to_string(), now + Duration::from_millis(wait));
                wait
            };
            if wait_ms > 0 {
                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            }
            let display_order = *order.get(plat.key()).unwrap_or(&0);
            let group = plat.group().to_string();
            // probe_one é 'static: todos os params são owned (String/Arc).
            let owned_user = user.clone();
            let key = plat.key();
            let probe = probe_one(
                plat,
                &owned_user,
                base_client,
                rotator,
                &router,
                &discord_tokens,
                &tiktok_sessions,
                &cfg,
            )
            .await;
            // Guarda no cache: não-livres permanentes (TTL longo) +
            // rate/error/unknown transitórios (10min, evita re-queimar o
            // mesmo nick rateado dentro do bulk). Available nunca entra.
            if let Some(ref p) = probe {
                if !p.cached {
                    cache.put(&p.norm, key, p.result, p.latency_ms).await;
                }
            }
            let class = match probe {
                None => ProbeClass::Ignore,
                Some(p) => classify_probe(p, &group, &owned_user, display_order),
            };
            Some(class) as Option<ProbeClass>
        });
    }

    let mut frees = 0usize;
    let mut warns = 0usize;
    let mut rates = 0usize;
    let mut errors = 0usize;
    let mut unknowns = 0usize;
    while let Some(res) = set.join_next().await {
        // Cancelado: aborta o restante sem contar; quem já terminou fica.
        if cancel.load(Ordering::Relaxed) {
            set.abort_all();
            break;
        }
        let Ok(Some(class)) = res else { continue };
        checked += 1;
        match class {
            ProbeClass::Hit(r) => {
                hits += 1;
                on_hit(r);
            }
            ProbeClass::Free(f) => {
                frees += 1;
                on_free(f);
            }
            ProbeClass::Warn(w) => {
                warns += 1;
                match w.reason.as_str() {
                    "rate" => rates += 1,
                    "error" => errors += 1,
                    _ => unknowns += 1,
                }
                on_warn(w);
            }
            ProbeClass::Ignore => {}
        }
        on_checked(checked);
    }
    (hits, frees, checked, warns, rates, errors, unknowns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_parses_scope_and_secs() {
        let (scope, secs) = parse_retry_after("retry_after scope=shared 985.0s").unwrap();
        assert_eq!(scope, "shared");
        assert!((secs - 985.0).abs() < f64::EPSILON);
        assert!(parse_retry_after("taken:true").is_none());
        assert!(parse_retry_after("waf").is_none());
    }

    #[test]
    fn bucket_strips_source_prefix() {
        assert_eq!(backoff_bucket("mf:discord"), "discord");
        assert_eq!(backoff_bucket("wm:github"), "github");
        assert_eq!(backoff_bucket("github"), "github");
    }

    #[test]
    fn shared_long_arms_user_short_does_not() {
        reset_backoff_for_tests();
        // Quorum: 1-2 eventos isolados NÃO param o mundo (só cooldown do par).
        note_ratelimited("testscope-quorum1", "retry_after scope=shared 900.0s");
        assert_eq!(backoff_remaining(&backoff_bucket("testscope-quorum1")), 0);
        note_ratelimited("testscope-quorum1", "retry_after scope=shared 900.0s");
        assert_eq!(backoff_remaining(&backoff_bucket("testscope-quorum1")), 0);
        // 3º evento no mesmo bucket dentro da janela = quorum, arma.
        note_ratelimited("testscope-quorum1", "retry_after scope=shared 900.0s");
        assert!(backoff_remaining(&backoff_bucket("testscope-quorum1")) > 800);
        // escopo user e retry curto: ruído local, nunca arma
        note_ratelimited("testscope-user", "retry_after scope=user 900.0s");
        note_ratelimited("testscope-user", "retry_after scope=user 900.0s");
        note_ratelimited("testscope-user", "retry_after scope=user 900.0s");
        assert_eq!(backoff_remaining(&backoff_bucket("testscope-user")), 0);
        note_ratelimited("testscope-short", "retry_after scope=shared 5.0s");
        note_ratelimited("testscope-short", "retry_after scope=shared 5.0s");
        note_ratelimited("testscope-short", "retry_after scope=shared 5.0s");
        assert_eq!(backoff_remaining(&backoff_bucket("testscope-short")), 0);
        note_ratelimited("testscope-waf", "waf");
        assert_eq!(backoff_remaining(&backoff_bucket("testscope-waf")), 0);
        reset_backoff_for_tests();
    }
}
