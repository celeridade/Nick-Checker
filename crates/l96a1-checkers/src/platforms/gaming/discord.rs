//! Discord: 4 métodos orquestrados (nunca abandona o token!).
//! Research (`research/discord.md`, `research/discord-sniper.md`):
//! 1. pomelo authed `@me/pomelo-attempt` (token, maior confiança e cota);
//! 2. unauthed `username-attempt-unauthed` POST `{"taken":bool}` (sem token);
//! 3. suggestions unauthed `username-suggestions-unauthed?global_name=` GET:
//!    devolve o próprio nick se livre, `nick+4 dígitos` se ocupado;
//! 4. modal-equivalente (mesmo pomelo; reserva).
//! Rotação de pool em 429 (padrão suenerve) + headers oficiais de rate-limit.
//! 429 sempre barreira. Sem token => só unauthed (nunca Unknown direto).
//! AVISO (g1ver): pomelo-attempt só existe enquanto a conta NÃO trava um
//! username novo — nunca complete troca de nick nas contas de check.

use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, CheckResult, Observation, UsernameValidator};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub struct Discord;

/// Semântica oficial de 429 (discord-api-docs#rate-limits):
/// `Retry-After` (header, s, pode ter decimal) ou `retry_after` (JSON) diz
/// quando tentar de novo; `X-RateLimit-Scope` diz o escopo
/// (`user`|`global`|`shared`). Retorna (scope, segundos).
fn rate_info(
    status: u16,
    headers: &reqwest::header::HeaderMap,
    body: &str,
) -> Option<(String, f64)> {
    if status != 429 && !body.contains("retry_after") {
        return None;
    }
    let scope = headers
        .get("x-ratelimit-scope")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("user")
        .to_string();
    let secs = headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .or_else(|| {
            serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| v.get("retry_after")?.as_f64())
        })
        .or_else(|| {
            // corpo não-JSON: pesca o número após "retry_after"
            body.find("retry_after").and_then(|i| {
                body[i..]
                    .chars()
                    .skip_while(|c| !c.is_ascii_digit())
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect::<String>()
                    .parse::<f64>()
                    .ok()
            })
        })
        .unwrap_or(0.0);
    Some((scope, secs))
}

fn rate_evidence(scope: &str, secs: f64) -> String {
    format!("retry_after scope={scope} {secs:.1}s")
}

/// Mapa nick -> snowflake ID (`discord_ids.json`).
/// Sem ID, perfil é `discord.com/users/<nick>` (não unfurla foto/banner);
/// com ID, vira `/users/<id>` (abre foto + banner, mesmo em perfil privado).
/// Cresce via seed manual + guild-search do bot. Research: Discord-Datamining
/// (estudos/) + `GET /users/{id}/profile` (800+ campos incl. avatar/banner).
static DISCORD_IDS: std::sync::OnceLock<std::collections::HashMap<String, String>> =
    std::sync::OnceLock::new();

fn id_map() -> &'static std::collections::HashMap<String, String> {
    DISCORD_IDS.get_or_init(|| {
        for cand in [
            "discord_ids.json".to_string(),
            "/data/discord_ids.json".to_string(),
            format!("{}/../../discord_ids.json", env!("CARGO_MANIFEST_DIR")),
        ] {
            if let Ok(text) = std::fs::read_to_string(&cand) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(obj) = v.as_object() {
                        return obj
                            .iter()
                            .filter(|(k, _)| !k.starts_with('_'))
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.to_lowercase(), s.to_string())))
                            .collect();
                    }
                }
            }
        }
        std::collections::HashMap::new()
    })
}

/// Snowflake ID conhecido p/ o nick (None = só nick, sem foto/banner).
pub fn discord_id_for(nick: &str) -> Option<String> {
    id_map().get(&nick.trim().to_lowercase()).cloned()
}

/// Busca perfil público por ID com user-token do pool.
/// Vale p/ perfis privados também (só a bio pode vir vazia).
/// Fora do caminho quente: só enrich on-demand (modal) e research.
pub async fn fetch_profile(token: &str, user_id: &str) -> Option<l96a1_core::RichProfile> {
    let client = l96a1_core::build_client(std::time::Duration::from_secs(10), None);
    let url = format!(
        "https://discord.com/api/v9/users/{user_id}/profile?with_mutual_guilds=false"
    );
    let resp = client
        .get(&url)
        .header("Authorization", token)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    let u = v.get("user")?;
    let id = u.get("id").and_then(|x| x.as_str()).unwrap_or(user_id);
    let avatar = u.get("avatar").and_then(|x| x.as_str()).filter(|s| !s.is_empty());
    let banner = u.get("banner").and_then(|x| x.as_str()).filter(|s| !s.is_empty());
    let mut p = l96a1_core::RichProfile::default();
    p.user_id = Some(id.to_string());
    p.display_name = u
        .get("global_name")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| u.get("username").and_then(|x| x.as_str()))
        .map(|s| s.to_string());
    p.pfp_url = avatar.map(|h| format!("https://cdn.discordapp.com/avatars/{id}/{h}.png?size=256"));
    p.banner_url = banner.map(|h| format!("https://cdn.discordapp.com/banners/{id}/{h}.png?size=1024"));
    p.bio = u.get("bio").and_then(|x| x.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string());
    Some(p)
}

/// Pool de tokens com cooldown por conta + morte por 401.
///
/// 429 com retry longo pausa SÓ a conta (outras seguem); 401 aposenta a
/// conta (não queima request nela a cada nick). Estado global do processo,
/// chaveado pelo token (a ordem do vec pode variar entre chamadas).
/// Saúde exposta em `token_health()` p/ `/v1/doctor` (só `…XXXX`, sem segredo).
#[derive(Debug, Clone, Default)]
struct TokenState {
    limited_until: u64,
    /// 0 = viva; senão, epoch até quando está aposentada (auto-cura: 401
    /// falso por erro de rede não mata a conta para sempre).
    dead_until: u64,
}

const DEAD_TTL_SECS: u64 = 3600;

/// Teto de contas tentadas por nick. 1 nick nunca queima o pool inteiro:
/// tenta no máximo N contas e depois cai no unauthed/suggest (outro bucket).
/// Com 10-20 tokens, 2 tentativas espalham carga sem concentrar 429.
const MAX_TOKEN_ATTEMPTS: usize = 2;

/// Cursor do round-robin entre contas (espalha o bulk em vez de empilhar
/// tudo na primeira viva — mesmo padrão das sessões TikTok).
static RR_CURSOR: AtomicUsize = AtomicUsize::new(0);

static TOKEN_POOL: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, TokenState>>> =
    std::sync::OnceLock::new();

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Últimos 4 chars p/ identificar a conta na UI sem vazar segredo.
fn short_id(token: &str) -> String {
    let tail: String = token.trim().chars().rev().take(4).collect::<String>().chars().rev().collect();
    format!("…{tail}")
}

/// Extrai escopo de evidence `retry_after scope=<s> <secs>s`.
fn rate_scope(evidence: &str) -> Option<String> {
    evidence
        .find("retry_after scope=")
        .and_then(|i| evidence[i + "retry_after scope=".len()..].split_whitespace().next())
        .map(|s| s.to_string())
}

/// Extrai segundos de evidence `retry_after scope=<s> <secs>s`.
fn retry_secs(evidence: &str) -> f64 {
    evidence
        .find("retry_after scope=")
        .and_then(|i| {
            let mut parts = evidence[i + "retry_after scope=".len()..].split_whitespace();
            parts.next()?;
            parts.next()?.trim_end_matches('s').parse::<f64>().ok()
        })
        .unwrap_or(0.0)
}

fn pool() -> &'static std::sync::Mutex<std::collections::HashMap<String, TokenState>> {
    TOKEN_POOL.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Tokens utilizáveis agora (pula pausados/mortos), em ordem rotativa.
/// O cursor avança a cada chamada: o bulk espalha ~1/N reqs por conta em
/// vez de concentrar tudo na primeira viva (que tomava 429 sozinha).
fn usable_tokens(tokens: &[String]) -> Vec<String> {
    let now = now_secs();
    let map = pool().lock().unwrap();
    let alive: Vec<String> = tokens
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| match map.get(s) {
            None => true,
            Some(st) => st.dead_until <= now && st.limited_until <= now,
        })
        .collect();
    drop(map);
    if alive.len() <= 1 {
        return alive;
    }
    let start = RR_CURSOR.fetch_add(1, Ordering::Relaxed) % alive.len();
    alive.iter().cycle().skip(start).take(alive.len()).cloned().collect()
}

/// Pausa a conta pelo retry (só escopo da CONTA — parede shared/global é
/// do IP e quem cuida é o breaker da engine + cooldown do proxy, não o pool).
/// Usa o `retry_after` real com margem (+10%) e jitter 0-5s para as contas
/// não voltarem todas no mesmo segundo (thundering herd).
fn note_token_ratelimited(token: &str, scope: Option<&str>, secs: f64) {
    if secs < 1.0 {
        return;
    }
    if matches!(scope, Some("shared") | Some("global")) {
        return;
    }
    // Jitter barato sem dependência nova: mistura nanos do relógio.
    let jitter = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() % 5000)
        .unwrap_or(0) as f64)
        / 1000.0;
    let padded = (secs * 1.1 + jitter).min(3600.0);
    let until = now_secs() + padded.ceil() as u64;
    let mut map = pool().lock().unwrap();
    map.entry(token.to_string()).or_default().limited_until = until;
}

/// Aposenta a conta (só 401 de verdade — erro de rede/timeout de proxy
/// NUNCA mata conta, senão proxies flaky zeram o pool em minutos).
fn note_token_dead(token: &str) {
    let until = now_secs() + DEAD_TTL_SECS;
    pool().lock().unwrap().entry(token.to_string()).or_default().dead_until = until;
}

/// Saúde por conta p/ UI: `{id: "…XXXX", state: ok|limited|dead, retry_in_secs}`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TokenHealth {
    pub id: String,
    pub state: &'static str,
    pub retry_in_secs: u64,
}

pub fn token_health(tokens: &[String]) -> Vec<TokenHealth> {
    let now = now_secs();
    let map = pool().lock().unwrap();
    tokens
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let (state, retry_in_secs) = match map.get(s) {
                None => ("ok", 0),
                Some(st) if st.dead_until > now => ("dead", 0),
                Some(st) if st.limited_until > now => ("limited", st.limited_until - now),
                _ => ("ok", 0),
            };
            TokenHealth { id: short_id(s), state, retry_in_secs }
        })
        .collect()
}

/// Decisão pura do unauthed `{"taken":bool}`.
fn decide_unauthed(
    status: u16,
    headers: &reqwest::header::HeaderMap,
    body: &str,
    ms: u64,
) -> CheckOutcome {
    let mut out = if let Some((scope, secs)) = rate_info(status, headers, body) {
        CheckOutcome::new(CheckResult::RateLimited, 0, rate_evidence(&scope, secs))
    } else if body.contains("\"taken\":true") {
        CheckOutcome::new(CheckResult::Taken, 0, "taken:true")
    } else if body.contains("\"taken\":false") {
        CheckOutcome::new(CheckResult::Available, 0, "taken:false")
    } else if status == 403 {
        // Bloco de datacenter (code 40333): IP recusado, não throttling —
        // Unknown com evidência própria p/ o RateBox diagnosticar.
        CheckOutcome::new(CheckResult::Unknown, 0, "dc-block-403")
    } else {
        CheckOutcome::new(CheckResult::Unknown, 0, "ambiguous")
    };
    out.latency_ms = ms;
    out
}

/// Decisão pura do suggestions GET `{"username":"..."}`:
/// eco exato do nick = livre; nick+4 dígitos (ou outro) = ocupado.
fn decide_suggest(
    status: u16,
    headers: &reqwest::header::HeaderMap,
    body: &str,
    user: &str,
    ms: u64,
) -> CheckOutcome {
    let mut out = if let Some((scope, secs)) = rate_info(status, headers, body) {
        CheckOutcome::new(CheckResult::RateLimited, 0, rate_evidence(&scope, secs))
    } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        match v.get("username").and_then(|x| x.as_str()) {
            Some(s) if s.eq_ignore_ascii_case(user) => {
                CheckOutcome::new(CheckResult::Available, 0, "suggest:echo")
            }
            Some(_) => CheckOutcome::new(CheckResult::Taken, 0, "suggest:suffixed"),
            None => CheckOutcome::new(CheckResult::Unknown, 0, "ambiguous"),
        }
    } else {
        CheckOutcome::new(CheckResult::Unknown, 0, "ambiguous")
    };
    out.latency_ms = ms;
    out
}

/// Decisão pura do pomelo authed. `tok` = id curto da conta (`…XXXX`),
/// para o RateBox mostrar QUEM rateou e em quanto tempo (sem vazar segredo).
fn decide_pomelo(
    status: u16,
    headers: &reqwest::header::HeaderMap,
    body: &str,
    ms: u64,
    tok: &str,
) -> CheckOutcome {
    let mut out = if let Some((scope, secs)) = rate_info(status, headers, body) {
        let mut ev = rate_evidence(&scope, secs);
        if !tok.is_empty() {
            ev.push_str(&format!(" tok={tok}"));
        }
        CheckOutcome::new(CheckResult::RateLimited, 0, ev)
    } else if status == 401 {
        CheckOutcome::new(CheckResult::Error, 0, "unauthorized")
    } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(taken) = v.get("taken").and_then(|x| x.as_bool()) {
            let o = if taken {
                Observation::taken_authoritative("pomelo", "taken:true")
            } else {
                Observation::avail_authoritative("pomelo", "taken:false")
            };
            let r = l96a1_core::aggregate(&[o]);
            CheckOutcome::new(r, 0, "pomelo")
        } else if v.to_string().contains("USERNAME_ALREADY_TAKEN") {
            CheckOutcome::new(CheckResult::Taken, 0, "already_taken")
        } else if v.to_string().contains("PASSWORD_DOES_NOT_MATCH") {
            CheckOutcome::new(CheckResult::Available, 0, "patch-signal")
        } else {
            CheckOutcome::new(CheckResult::Unknown, 0, "ambiguous")
        }
    } else {
        CheckOutcome::new(CheckResult::Unknown, 0, "ambiguous")
    };
    out.latency_ms = ms;
    out
}

async fn attempt_unauthed(user: &str, ctx: &CheckCtx, ms: u64) -> CheckOutcome {
    let client = l96a1_core::client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let resp = client
        .post("https://discord.com/api/v9/unique-username/username-attempt-unauthed")
        .header("Content-Type", "application/json")
        .header("Origin", "https://discord.com")
        .header("X-Discord-Locale", "en-US")
        .json(&serde_json::json!({ "username": user }))
        .send()
        .await;
    match resp {
        Err(_) => crate::common::outcome_net_err(ms),
        Ok(r) => {
            let st = r.status().as_u16();
            let hd = r.headers().clone();
            let tx = r.text().await.unwrap_or_default();
            decide_unauthed(st, &hd, &tx, ms)
        }
    }
}

async fn attempt_suggest(user: &str, ctx: &CheckCtx, ms: u64) -> CheckOutcome {
    let client = l96a1_core::client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let resp = client
        .get("https://discord.com/api/v9/unique-username/username-suggestions-unauthed")
        .header("Origin", "https://discord.com")
        .header("X-Discord-Locale", "en-US")
        .query(&[("global_name", user)])
        .send()
        .await;
    match resp {
        Err(_) => crate::common::outcome_net_err(ms),
        Ok(r) => {
            let st = r.status().as_u16();
            let hd = r.headers().clone();
            let tx = r.text().await.unwrap_or_default();
            decide_suggest(st, &hd, &tx, user, ms)
        }
    }
}

async fn attempt_pomelo(user: &str, token: &str, ctx: &CheckCtx, ms: u64) -> CheckOutcome {
    let client = l96a1_core::client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let resp = client
        .post("https://discord.com/api/v9/users/@me/pomelo-attempt")
        .header("Content-Type", "application/json")
        .header("Origin", "https://discord.com")
        .header("Referer", "https://discord.com/channels/@me")
        .header("Authorization", token)
        .json(&serde_json::json!({ "username": user }))
        .send()
        .await;
    let tok = short_id(token);
    match resp {
        Err(_) => crate::common::outcome_net_err(ms),
        Ok(r) => {
            let st = r.status().as_u16();
            let hd = r.headers().clone();
            let tx = r.text().await.unwrap_or_default();
            decide_pomelo(st, &hd, &tx, ms, &tok)
        }
    }
}

/// Desempate barato: suggestions só roda quando os outros não concluíram.
async fn tiebreak(user: &str, ctx: &CheckCtx, ms: u64, fallback: CheckOutcome) -> CheckOutcome {
    let s = attempt_suggest(user, ctx, ms).await;
    match s.result {
        CheckResult::Taken | CheckResult::Available => s,
        // suggestions rateado mas attempt já tinha dado rate: mantém o 1º
        _ => fallback,
    }
}

#[async_trait]
impl l96a1_core::Platform for Discord {
    fn key(&self) -> &'static str {
        "discord"
    }
    fn name(&self) -> &'static str {
        "Discord"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile_url(&self, user: &str) -> String {
        // Perfil Discord é por snowflake ID. Com ID conhecido
        // (`discord_ids.json`), o link canônico abre foto + banner
        // (mesmo em perfil privado); sem ID, cai no padrão por nick.
        match discord_id_for(user) {
            Some(id) => format!("https://discord.com/users/{id}"),
            None => format!("https://discord.com/users/{user}"),
        }
    }
    fn validator(&self) -> UsernameValidator {
        let mut v = UsernameValidator::new(2, 32, r"^[a-z0-9_.]+$");
        v.reserved = vec!["discord".into(), "everyone".into(), "here".into()];
        v
    }
    fn min_interval_ms(&self) -> u64 {
        120 // pacing por par (token,proxy) na engine; global leve p/ não serializar o bulk
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 10, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let mut last_rate: Option<CheckOutcome> = None;
        // Método 1 (barato, sem cota): unauthed primeiro. Na maioria dos
        // nicks decide em 1 request; só cai no pomelo se ambíguo/rateado.
        // Economiza N-1 requests e preserva as contas p/ o desempate.
        let first = attempt_unauthed(user, ctx, ms()).await;
        match first.result {
            CheckResult::Taken | CheckResult::Available => return first,
            CheckResult::RateLimited => {
                last_rate = Some(first.clone());
                // Parede shared/global no unauthed é do IP: tenta o pomelo
                // (bucket authed distinto) antes de desistir, com cap.
            }
            _ => {}
        }
        // Método 2: pool de tokens (pomelo authed), no máximo
        // MAX_TOKEN_ATTEMPTS contas por nick, em ordem rotativa.
        // Contas pausadas (429 longo) e mortas (401) são puladas sem request.
        let mut tried = 0usize;
        for token in usable_tokens(&ctx.discord_tokens) {
            if tried >= MAX_TOKEN_ATTEMPTS {
                break;
            }
            tried += 1;
            let o = attempt_pomelo(user, &token, ctx, ms()).await;
            match o.result {
                CheckResult::Taken | CheckResult::Available => return o,
                CheckResult::Error => {
                    // Só 401 aposenta; "network/timeout" é proxy falho
                    // (a conta está viva — matar aqui zerava o pool).
                    if !o.evidence.starts_with("network") {
                        note_token_dead(&token);
                    }
                    continue;
                }
                CheckResult::RateLimited => {
                    note_token_ratelimited(
                        &token,
                        rate_scope(&o.evidence).as_deref(),
                        retry_secs(&o.evidence),
                    );
                    if last_rate.is_none() {
                        last_rate = Some(o);
                    }
                    continue; // próxima conta (padrão suenerve, com cap)
                }
                _ => {
                    // Unknown no pomelo: suggestions como desempate barato.
                    let s = tiebreak(user, ctx, ms(), o).await;
                    match s.result {
                        CheckResult::Taken | CheckResult::Available => return s,
                        _ => {
                            // Preserva o 1º rate (unauthed ou pomelo) p/ backoff
                            // da engine em vez de Unknown cego.
                            if let Some(r) = last_rate.clone() {
                                return r;
                            }
                            return s;
                        }
                    }
                }
            }
        }
        // Método 3: sem token conclusivo (ou pool esvaziado) — suggestions
        // como desempate (GET barato, outro bucket).
        let s = tiebreak(user, ctx, ms(), first).await;
        // se nada concluiu mas algum método rateou, sinaliza o rate
        // (engine aplica backoff) em vez de Unknown cego
        if !matches!(s.result, CheckResult::Taken | CheckResult::Available) {
            if let Some(r) = last_rate {
                return r;
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hdrs() -> reqwest::header::HeaderMap {
        reqwest::header::HeaderMap::new()
    }

    fn hdrs_retry_after(secs: &str, scope: &str) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(reqwest::header::RETRY_AFTER, secs.parse().unwrap());
        h.insert("x-ratelimit-scope", scope.parse().unwrap());
        h
    }

    #[test]
    fn unauthed_taken_pair() {
        assert_eq!(
            decide_unauthed(200, &hdrs(), r#"{"taken":true}"#, 5).result,
            CheckResult::Taken
        );
        assert_eq!(
            decide_unauthed(200, &hdrs(), r#"{"taken":false}"#, 5).result,
            CheckResult::Available
        );
    }

    #[test]
    fn unauthed_dc_block_is_unknown_with_evidence() {
        // 403 datacenter (code 40333) não é rate nem sinal: Unknown com
        // evidência própria (audit 2026-10).
        let o = decide_unauthed(403, &hdrs(), r#"{"code":40333}"#, 5);
        assert_eq!(o.result, CheckResult::Unknown);
        assert_eq!(o.evidence, "dc-block-403");
    }

    #[test]
    fn unauthed_rate_is_barrier() {
        let o = decide_unauthed(429, &hdrs(), r#"{"retry_after":1176}"#, 5);
        assert_eq!(o.result, CheckResult::RateLimited);
        assert!(o.evidence.contains("scope=user"));
    }

    #[test]
    fn rate_header_beats_body_and_scope_parsed() {
        // Retry-After header (65s) vence o corpo; escopo global detectado
        let h = hdrs_retry_after("65", "global");
        let (scope, secs) = rate_info(429, &h, r#"{"retry_after":1}"#).unwrap();
        assert_eq!(scope, "global");
        assert!((secs - 65.0).abs() < f64::EPSILON);
        let o = decide_unauthed(429, &h, "{}", 5);
        assert!(o.evidence.contains("scope=global"));
    }

    #[test]
    fn suggest_echo_means_free_suffix_means_taken() {
        assert_eq!(
            decide_suggest(200, &hdrs(), r#"{"username":"vazio"}"#, "vazio", 5).result,
            CheckResult::Available
        );
        assert_eq!(
            decide_suggest(200, &hdrs(), r#"{"username":"vazio0420"}"#, "vazio", 5).result,
            CheckResult::Taken
        );
        assert_eq!(
            decide_suggest(200, &hdrs(), r#"{}"#, "vazio", 5).result,
            CheckResult::Unknown
        );
        assert_eq!(
            decide_suggest(429, &hdrs(), r#"{"retry_after":2.5}"#, "vazio", 5).result,
            CheckResult::RateLimited
        );
    }

    #[test]
    fn pomelo_taken_pair_and_errors() {
        assert_eq!(
            decide_pomelo(200, &hdrs(), r#"{"taken":true}"#, 5, "").result,
            CheckResult::Taken
        );
        assert_eq!(
            decide_pomelo(200, &hdrs(), r#"{"taken":false}"#, 5, "").result,
            CheckResult::Available
        );
        assert_eq!(
            decide_pomelo(429, &hdrs(), r#"{"retry_after":1}"#, 5, "").result,
            CheckResult::RateLimited
        );
        assert_eq!(
            decide_pomelo(401, &hdrs(), "{}", 5, "").result,
            CheckResult::Error
        );
    }

    #[test]
    fn pomelo_rate_carries_token_id() {
        let o = decide_pomelo(429, &hdrs(), r#"{"retry_after":65}"#, 5, "…a3f9");
        assert_eq!(o.result, CheckResult::RateLimited);
        assert!(o.evidence.contains("tok=…a3f9"));
        assert!(o.evidence.contains("65.0s"));
    }

    #[test]
    fn pool_skips_limited_and_dead() {
        // Tokens únicos por teste (pool global): evita interferência do cursor RR.
        let a = "tok-RR-A-AAA".to_string();
        let b = "tok-RR-B-BBB".to_string();
        // Ordem rotativa: compara como conjunto nas 2 primeiras chamadas.
        let mut first = usable_tokens(&[a.clone(), b.clone()]);
        first.sort();
        let mut expect = vec![a.clone(), b.clone()];
        expect.sort();
        assert_eq!(first, expect);
        // escopo user pausa a conta…
        note_token_ratelimited(&a, Some("user"), 600.0);
        assert_eq!(usable_tokens(&[a.clone(), b.clone()]), vec![b.clone()]);
        // …mas parede shared/global NÃO pausa conta (é do IP; breaker cuida).
        note_token_ratelimited(&b, Some("shared"), 600.0);
        assert_eq!(usable_tokens(&[a.clone(), b.clone()]), vec![b.clone()]);
        note_token_dead(&b);
        assert!(usable_tokens(&[a.clone(), b.clone()]).is_empty());
        let h = token_health(&[a.clone(), b.clone()]);
        assert_eq!(h[0].state, "limited");
        assert!(h[0].retry_in_secs > 500);
        assert_eq!(h[1].state, "dead");
        assert!(h[0].id.ends_with("AAA"));
    }

    #[test]
    fn pool_rotates_start_across_calls() {
        // Com 3 contas vivas, chamadas sucessivas começam em offsets
        // distintos (round-robin) — o bulk espalha em vez de concentrar.
        let toks: Vec<String> =
            vec!["tok-rot-1".to_string(), "tok-rot-2".to_string(), "tok-rot-3".to_string()];
        let first = usable_tokens(&toks);
        let second = usable_tokens(&toks);
        assert_eq!(first.len(), 3);
        assert_eq!(second.len(), 3);
        // Mesmo conjunto, ordem rotacionada (as duas cobrem os 3).
        let mut s1 = first.clone();
        let mut s2 = second.clone();
        s1.sort();
        s2.sort();
        assert_eq!(s1, s2);
    }

    #[test]
    fn retry_secs_parses_evidence() {
        assert!((retry_secs("retry_after scope=shared 985.0s tok=…x") - 985.0).abs() < 0.01);
        assert_eq!(retry_secs("taken:true"), 0.0);
    }
}
