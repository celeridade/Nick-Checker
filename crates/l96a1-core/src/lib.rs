//! l96a1-core — tipos centrais, validador, agregador de evidencias.
//! Regra ouro: nunca classificar sem confirmacao positiva.

use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Resultado possivel de um check. Qualquer ambiguidade => Unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckResult {
    Available,
    Taken,
    Private,
    /// Conta existe mas está banida/suspensa (nunca claimável).
    Banned,
    RateLimited,
    Error,
    Unknown,
}

impl CheckResult {
    pub fn emoji(self) -> &'static str {
        match self {
            CheckResult::Available => "✅",
            CheckResult::Taken => "🔴",
            CheckResult::Private => "🔒",
            CheckResult::Banned => "🚫",
            CheckResult::RateLimited => "⚠️",
            CheckResult::Error => "❌",
            CheckResult::Unknown => "❓",
        }
    }

    /// Simbolo geometrico minimalista para o HUD tatico.
    pub fn symbol(self) -> &'static str {
        match self {
            CheckResult::Available => "●",
            CheckResult::Taken => "·",
            CheckResult::Private => "◆",
            CheckResult::Banned => "✕",
            CheckResult::RateLimited => "!",
            CheckResult::Error => "✕",
            CheckResult::Unknown => "?",
        }
    }

    /// Tag curta em caixa-alta (alinhada p/ tabelas e telemetria HUD).
    pub fn tag(self) -> &'static str {
        match self {
            CheckResult::Available => "HIT",
            CheckResult::Taken => "TAKEN",
            CheckResult::Private => "PRIV",
            CheckResult::Banned => "BAN",
            CheckResult::RateLimited => "RATE",
            CheckResult::Error => "ERR",
            CheckResult::Unknown => "UNK",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CheckResult::Available => "avail",
            CheckResult::Taken => "taken",
            CheckResult::Private => "private",
            CheckResult::Banned => "banned",
            CheckResult::RateLimited => "rate",
            CheckResult::Error => "error",
            CheckResult::Unknown => "unknown",
        }
    }
}

/// Uma observacao (sinal) coletada durante o check.
#[derive(Debug, Clone)]
pub struct Observation {
    pub method: &'static str,
    pub taken: bool,
    pub available: bool,
    pub barrier: bool,
    pub authoritative: bool,
    pub banned: bool,
    pub note: String,
}

impl Observation {
    pub fn taken_authoritative(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: true, available: false, barrier: false, authoritative: true, banned: false, note: note.into() }
    }
    pub fn taken_strong(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: true, available: false, barrier: false, authoritative: false, banned: false, note: note.into() }
    }
    pub fn avail_authoritative(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: false, available: true, barrier: false, authoritative: true, banned: false, note: note.into() }
    }
    pub fn avail_strong(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: false, available: true, barrier: false, authoritative: false, banned: false, note: note.into() }
    }
    pub fn barrier(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: false, available: false, barrier: true, authoritative: false, banned: false, note: note.into() }
    }
    pub fn banned_authoritative(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: false, available: false, barrier: false, authoritative: true, banned: true, note: note.into() }
    }
    pub fn banned_strong(method: &'static str, note: impl Into<String>) -> Self {
        Self { method, taken: false, available: false, barrier: false, authoritative: false, banned: true, note: note.into() }
    }
}

/// Agregador deterministico: AUTHORITATIVE vence; AVAILABLE exige sinal
/// autoritativo OU >=2 fortes independentes e zero barreiras.
pub fn aggregate(observations: &[Observation]) -> CheckResult {
    let mut auth_taken = false;
    let mut auth_avail = false;
    let mut strong_taken = 0u32;
    let mut strong_avail = 0u32;
    let mut auth_banned = false;
    let mut strong_banned = 0u32;
    let mut barrier = false;
    let mut private = false;

    for o in observations {
        if o.barrier {
            barrier = true;
            continue;
        }
        if o.banned {
            if o.authoritative {
                auth_banned = true;
            } else {
                strong_banned += 1;
            }
            continue;
        }
        if o.note.contains("private") {
            private = true;
        }
        match (o.taken, o.available, o.authoritative) {
            (true, _, true) => auth_taken = true,
            (_, true, true) => auth_avail = true,
            (true, _, false) => strong_taken += 1,
            (_, true, false) => strong_avail += 1,
            _ => {}
        }
    }

    if (auth_taken && auth_avail) || (strong_taken > 0 && strong_avail > 0 && !auth_taken && !auth_avail) {
        return CheckResult::Unknown; // conflito
    }
    // Banido com sinal de disponível junto = chão instável: Unknown.
    // Banido vence Taken/Private (conta existente mas não claimável).
    if (auth_banned || strong_banned > 0) && (auth_avail || strong_avail > 0) {
        return CheckResult::Unknown; // conflito
    }
    if auth_banned || strong_banned > 0 {
        return CheckResult::Banned;
    }
    if auth_taken || strong_taken > 0 {
        return if private { CheckResult::Private } else { CheckResult::Taken };
    }
    if !barrier && (auth_avail || strong_avail >= 2) {
        return CheckResult::Available;
    }
    if barrier {
        return CheckResult::RateLimited;
    }
    CheckResult::Unknown
}

/// Validador de username por plataforma (pre-filtro antes de qualquer I/O).
#[derive(Debug, Clone)]
pub struct UsernameValidator {
    pub min_length: usize,
    pub max_length: usize,
    pub regex: Regex,
    pub reserved: Vec<String>,
    pub strip_leading_at: bool,
}

impl UsernameValidator {
    pub fn new(min: usize, max: usize, pattern: &str) -> Self {
        Self {
            min_length: min,
            max_length: max,
            regex: Regex::new(pattern).expect("regex invalida"),
            reserved: Vec::new(),
            strip_leading_at: true,
        }
    }

    pub fn normalize(&self, raw: &str) -> Option<String> {
        let mut u = raw.trim().to_string();
        if self.strip_leading_at && u.starts_with('@') {
            u.remove(0);
        }
        if u.len() < self.min_length || u.len() > self.max_length {
            return None;
        }
        let check = u.to_lowercase();
        if !self.regex.is_match(&check) {
            return None;
        }
        if self.reserved.iter().any(|r| r == &check) {
            return None;
        }
        Some(u)
    }
}

impl Default for UsernameValidator {
    fn default() -> Self {
        Self::new(1, 30, r"^[a-z0-9_.]+$")
    }
}

/// Perfil de velocidade/roteamento por plataforma.
/// Cada site tem sua velocidade ideal: youtube rapido, discord lento.
#[derive(Debug, Clone)]
pub struct PlatformProfile {
    /// Teto de concorrencia simultanea nesta plataforma.
    pub max_concurrent: usize,
    /// Proxy obrigatorio (rate agressivo / WAF).
    pub needs_proxy: bool,
    /// Mesmo proxy por dominio (sessoes/WAF com afinidade).
    pub sticky_proxy: bool,
}

impl Default for PlatformProfile {
    fn default() -> Self {
        Self { max_concurrent: 16, needs_proxy: false, sticky_proxy: false }
    }
}

/// Contexto de um check: client compartilhado + credenciais + proxy.
#[derive(Clone)]
pub struct CheckCtx {
    pub client: Arc<reqwest::Client>,
    pub proxy_url: Option<String>,
    /// Pool de tokens Discord (rotação em 429, padrão suenerve).
    /// Vazio = só métodos unauthed.
    pub discord_tokens: Vec<String>,
    /// Pool de sessões TikTok logadas (cookie-string por conta, uma por
    /// linha em `tiktok_sessions.txt`). Vazio = só métodos unauthed
    /// (oembed+page). Com sessões, o check autenticado
    /// (`/api/uniqueid/check/`) vira árbitro antes do fallback.
    pub tiktok_sessions: Vec<String>,
    pub github_token: Option<String>,
    pub timeout: Duration,
}

/// Resultado de um check individual.
#[derive(Debug, Clone)]
pub struct CheckOutcome {
    pub result: CheckResult,
    pub latency_ms: u64,
    pub evidence: String,
}

impl CheckOutcome {
    pub fn new(result: CheckResult, latency_ms: u64, evidence: impl Into<String>) -> Self {
        Self { result, latency_ms, evidence: evidence.into() }
    }
}

/// Perfil enriquecido (API separada `/v1/enrich` — nunca no caminho quente do check).
/// Só dado público: avatar, banner, display, bio, contadores. Sem token sensível.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RichProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pfp_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followers: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub following: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posts: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(default)]
    pub emails: Vec<String>,
}

/// Mascaramento de proxy p/ logs/SSE (nunca vaza `user:pass@`).
/// `http://user:pass@1.2.3.4:8080` → `http://***:8080`; `None` → `direct`.
pub fn mask_proxy(proxy: Option<&str>) -> String {
    match proxy {
        None => "direct".to_string(),
        Some(p) => {
            let p = p.trim();
            if p.is_empty() {
                return "direct".to_string();
            }
            // Rótulos do router passam direto.
            if ["direct", "pool/direct", "sticky-proxy", "proxy-req"].contains(&p) {
                return p.to_string();
            }
            // Tenta extrair só esquema + porta, sem credencial/host.
            if let Some(after) = p.split("://").nth(1) {
                let host_port = after.split('@').next_back().unwrap_or(after);
                let port = host_port.split(':').next_back().unwrap_or("");
                let scheme = p.split("://").next().unwrap_or("http");
                if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
                    return format!("{scheme}://***:{port}");
                }
                return format!("{scheme}://***");
            }
            "***".to_string()
        }
    }
}

/// Trait de plataforma: um arquivo por site em `platforms/`.
#[async_trait]
pub trait Platform: Send + Sync {
    fn key(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn group(&self) -> &'static str;
    fn profile_url(&self, user: &str) -> String;
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::default()
    }
    /// Intervalo minimo entre requests (rate-limit local). 0 = sem limite.
    fn min_interval_ms(&self) -> u64 {
        0
    }
    /// Perfil de velocidade/roteamento (sobrescrito por plataforma).
    fn profile(&self) -> PlatformProfile {
        PlatformProfile::default()
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome;
}

/// Helpers de decisao status+body (padrao blackbird/sherlock).
pub fn decide_http(
    status: u16,
    body: &str,
    taken_markers: &[&str],
    avail_markers: &[&str],
    rate_markers: &[&str],
) -> Vec<Observation> {
    let mut obs = Vec::new();
    let lower = body.to_lowercase();
    if status == 429 || rate_markers.iter().any(|m| lower.contains(&m.to_lowercase())) {
        obs.push(Observation::barrier("http", "rate-limited"));
        return obs;
    }
    if status == 404 {
        obs.push(Observation::avail_authoritative("http", "404"));
    }
    for m in avail_markers {
        if body.contains(m) {
            obs.push(Observation::avail_strong("http", format!("marker:{m}")));
        }
    }
    for m in taken_markers {
        if body.contains(m) {
            obs.push(Observation::taken_strong("http", format!("marker:{m}")));
        }
    }
    if status == 200 && obs.is_empty() {
        // 200 sem marcador => nao decide (evita falso-positivo).
        obs.push(Observation::barrier("http", "ambiguous-200"));
    }
    obs
}

/// Encurta URL para o stream (`https://www.` e `https://` removidos).
pub fn short_link(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .to_string()
}

pub const CHROME_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Construtor do client global compartilhado (pool unico).
pub fn build_client(timeout: Duration, proxy_url: Option<&str>) -> reqwest::Client {
    build_client_with(timeout, proxy_url, reqwest::redirect::Policy::limited(5))
}

/// Variante sem seguir redirects (p/ controle manual + cookie-jar proprio).
pub fn build_client_nofollow(timeout: Duration, proxy_url: Option<&str>) -> reqwest::Client {
    build_client_with(timeout, proxy_url, reqwest::redirect::Policy::none())
}

fn build_client_with(
    timeout: Duration,
    proxy_url: Option<&str>,
    redirect: reqwest::redirect::Policy,
) -> reqwest::Client {
    let mut b = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(CHROME_UA)
        .pool_max_idle_per_host(50)
        .redirect(redirect);
    if let Some(p) = proxy_url {
        if let Ok(proxy) = reqwest::Proxy::all(p) {
            b = b.proxy(proxy);
        }
    }
    b.build().expect("falha ao criar http client")
}

static CLIENT_CACHE: std::sync::Mutex<Option<HashMap<String, Arc<reqwest::Client>>>> =
    std::sync::Mutex::new(None);

/// Client com proxy especifico (cacheado por url+timeout — reqwest so
/// aceita proxy no nivel do Client, nao por request).
pub fn client_for(timeout: Duration, proxy_url: Option<&str>) -> Arc<reqwest::Client> {
    let key = format!("{}|{}", timeout.as_secs(), proxy_url.unwrap_or("direct"));
    let mut guard = CLIENT_CACHE.lock().unwrap();
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(c) = map.get(&key) {
        return c.clone();
    }
    let c = Arc::new(build_client(timeout, proxy_url));
    map.insert(key, c.clone());
    c
}

/// Mapa de rate-limit local por plataforma (min interval).
#[derive(Default)]
pub struct LocalBuckets {
    last: HashMap<&'static str, std::time::Instant>,
}

impl LocalBuckets {
    pub async fn wait(&mut self, key: &'static str, min_ms: u64) {
        if min_ms == 0 {
            return;
        }
        let now = std::time::Instant::now();
        if let Some(prev) = self.last.get(key) {
            let elapsed = now.duration_since(*prev).as_millis() as u64;
            if elapsed < min_ms {
                tokio::time::sleep(Duration::from_millis(min_ms - elapsed)).await;
            }
        }
        self.last.insert(key, std::time::Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_enforces_gold_rule_and_precedence() {
        // Sem evidencias => Unknown
        assert_eq!(aggregate(&[]), CheckResult::Unknown);

        // 1 sinal forte de disponivel nao basta sozinho; exige 2 fortes ou 1 autoritativo
        assert_eq!(
            aggregate(&[Observation::avail_strong("page", "no-profile")]),
            CheckResult::Unknown
        );
        assert_eq!(
            aggregate(&[
                Observation::avail_strong("page", "no-profile"),
                Observation::avail_strong("api", "empty"),
            ]),
            CheckResult::Available
        );
        assert_eq!(
            aggregate(&[Observation::avail_authoritative("api", "404")]),
            CheckResult::Available
        );

        // Barreira bloqueia Available e retorna RateLimited quando nao ha Taken/Banned
        assert_eq!(
            aggregate(&[
                Observation::avail_authoritative("api", "404"),
                Observation::barrier("waf", "captcha"),
            ]),
            CheckResult::RateLimited
        );

        // Conflito entre Taken e Available => Unknown
        assert_eq!(
            aggregate(&[
                Observation::taken_authoritative("api", "200"),
                Observation::avail_authoritative("page", "404"),
            ]),
            CheckResult::Unknown
        );

        // Banned vence Taken/Private, mas conflita com Available
        assert_eq!(
            aggregate(&[
                Observation::banned_authoritative("api", "suspended"),
                Observation::taken_strong("page", "exists"),
            ]),
            CheckResult::Banned
        );
        assert_eq!(
            aggregate(&[
                Observation::banned_authoritative("api", "suspended"),
                Observation::avail_authoritative("api", "404"),
            ]),
            CheckResult::Unknown
        );

        // Nota com "private" converte Taken em Private
        assert_eq!(
            aggregate(&[Observation::taken_authoritative("api", "private-account")]),
            CheckResult::Private
        );
    }

    #[test]
    fn validator_normalizes_and_filters_reserved() {
        let mut v = UsernameValidator::new(3, 12, r"^[a-z0-9_]+$");
        v.reserved = vec!["admin".into(), "support".into()];

        assert_eq!(v.normalize("  @hikaru "), Some("hikaru".into()));
        assert_eq!(v.normalize("ab"), None); // curto demais
        assert_eq!(v.normalize("bad-char!"), None);
        assert_eq!(v.normalize("ADMIN"), None); // reservado
    }

    #[test]
    fn short_link_strips_scheme_and_www() {
        assert_eq!(short_link("https://www.instagram.com/abc/"), "instagram.com/abc/");
        assert_eq!(short_link("http://github.com/xyz"), "github.com/xyz");
    }
}
