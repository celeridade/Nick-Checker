//! l96a1-api — motor Rust como serviço HTTP para a web.
//!
//! Mantém 100% da velocidade do CLI (tokio + reqwest/rustls + Rotator +
//! HybridRouter + semáforos por plataforma). O Next.js só faz proxy/BFF.

mod checked;
mod enrich;
mod models;
mod scan;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json, Redirect,
    },
    routing::{get, post},
    Router,
};
use models::{HistoryEntry, JobStatus, PlatformInfo, ScanJob, SearchResult};
use scan::{dedupe_usernames, origin_of, probe_one, resolve_platforms, run_scan, EngineConfig};
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone)]
struct AppConfig {
    timeout_secs: u64,
    max_concurrent: usize,
    proxies_file: String,
    github_token: String,
    throttle_batch: usize,
    throttle_delay_secs: f64,
    warn_cap: usize,
    dataset_dir: String,
    webhook_url: String,
    webhook_mode: String,
    snipe_platforms: String,
    snipe_interval_secs: u64,
    watchlist_file: String,
    bot_token: String,
    bot_guild_id: u64,
    bot_prefix: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct SnipeStatus {
    running: bool,
    last_run: Option<String>,
    next_in_secs: u64,
    watchlist_size: usize,
    cycles: u64,
    hits_total: u64,
    last_hits: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct BotSetupSummary {
    ran_at: String,
    verify_channel_id: u64,
    channels: usize,
    log: Vec<String>,
}

impl AppConfig {
    fn from_env() -> Self {
        let timeout_secs = std::env::var("TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let max_concurrent = std::env::var("MAX_CONCURRENT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        let throttle_batch = std::env::var("THROTTLE_BATCH_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let throttle_delay_secs = std::env::var("THROTTLE_DELAY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30.0);
        // Teto de warns guardados por job (auditoria completa pede alto;
        // UI faz polling — default 500 mantém o tráfego leve).
        let warn_cap = std::env::var("WARN_CAP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500);
        Self {
            timeout_secs,
            max_concurrent,
            proxies_file: std::env::var("PROXIES_FILE").unwrap_or_else(|_| "proxies.txt".into()),
            github_token: std::env::var("GITHUB_TOKEN").unwrap_or_default(),
            throttle_batch,
            throttle_delay_secs,
            warn_cap,
            dataset_dir: std::env::var("DATASET_DIR").unwrap_or_else(|_| "dataset".into()),
            webhook_url: std::env::var("WEBHOOK_URL").unwrap_or_default(),
            webhook_mode: std::env::var("WEBHOOK_MODE").unwrap_or_else(|_| "batch".into()),
            snipe_platforms: std::env::var("SNIPE_PLATFORMS").unwrap_or_else(|_| "all".into()),
            snipe_interval_secs: std::env::var("SNIPE_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            watchlist_file: std::env::var("WATCHLIST_FILE").unwrap_or_else(|_| "watchlist.txt".into()),
            bot_token: std::env::var("BOT_TOKEN")
                .or_else(|_| std::env::var("DISCORD_BOT_TOKEN"))
                .unwrap_or_default(),
            bot_guild_id: std::env::var("BOT_GUILD_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            bot_prefix: std::env::var("BOT_PREFIX").unwrap_or_else(|_| "!".into()),
        }
    }

    fn engine(&self) -> EngineConfig {
        EngineConfig {
            timeout_secs: self.timeout_secs,
            max_concurrent: self.max_concurrent,
            github_token: self.github_token.clone(),
            throttle_batch: self.throttle_batch,
            throttle_delay_secs: self.throttle_delay_secs,
        }
    }
}

struct AppState {
    cfg: AppConfig,
    rotator: Arc<l96a1_proxy::Rotator>,
    discord_tokens: Vec<String>,
    /// Sessões TikTok logadas (cookie-string por conta). Vazio = checker
    /// roda só unauthed (oembed+page); com sessões, o
    /// `/api/uniqueid/check/` vira árbitro antes do fallback.
    tiktok_sessions: Vec<String>,
    version: String,
    jobs: Mutex<HashMap<String, ScanJob>>,
    history: Mutex<Vec<HistoryEntry>>,
    watchlist: Mutex<Vec<String>>,
    snipe: Mutex<SnipeStatus>,
    bot_setup: Mutex<Option<BotSetupSummary>>,
    enrich_cache: enrich::EnrichCache,
    /// Nicks não-livres já checados (evita re-check; limpa manual/TTL).
    checked_cache: Arc<checked::CheckedCache>,
    /// Flags de cancelamento por job (jobs via POST /v1/scans).
    cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

/// Resolve arquivo de estado (history/checked) independente do CWD:
/// 1) env explícito vale sempre; 2) primeiro candidato que já existe como
/// arquivo; 3) primeiro cujo diretório-pai existe; 4) o default como está.
/// (Mesma classe de bug do dataset: path relativo resolvia no CWD errado e
/// o .json "sumia" — era escrito em outra pasta.)
fn resolve_state_file(env_key: &str, default_name: &str) -> String {
    if let Ok(v) = std::env::var(env_key) {
        if !v.trim().is_empty() {
            return v;
        }
    }
    let repo = format!("{}/../../{}", env!("CARGO_MANIFEST_DIR"), default_name);
    let exe_side = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(default_name).to_string_lossy().to_string()));
    let mut cands = vec![
        format!("/data/{default_name}"),
        repo,
    ];
    if let Some(e) = exe_side {
        cands.push(e);
    }
    cands.push(default_name.to_string());
    for c in &cands {
        if std::path::Path::new(c).is_file() {
            return c.clone();
        }
    }
    for c in &cands {
        let parent = std::path::Path::new(c).parent();
        if parent.map(|p| p.as_os_str().is_empty() || p.is_dir()).unwrap_or(false) {
            return c.clone();
        }
    }
    default_name.to_string()
}

fn history_file() -> String {
    resolve_state_file("HISTORY_FILE", "history.json")
}

fn load_history(path: &str) -> Vec<HistoryEntry> {
    // Persiste entre restarts (era só memória e resetava no boot).
    // Tenta o configurado, depois /data/history.json (Docker), depois ./history.json.
    for cand in [
        path.to_string(),
        "/data/history.json".to_string(),
        "history.json".to_string(),
    ] {
        if let Ok(text) = std::fs::read_to_string(&cand) {
            if let Ok(v) = serde_json::from_str::<Vec<HistoryEntry>>(&text) {
                if !v.is_empty() || std::path::Path::new(&cand).exists() {
                    // Cap anti-abuso igual ao push (200).
                    return v.into_iter().rev().take(200).rev().collect();
                }
            }
        }
    }
    Vec::new()
}

fn save_history(path: &str, h: &[HistoryEntry]) {
    let p = if path.is_empty() { "history.json" } else { path };
    if let Some(parent) = std::path::Path::new(p).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    // try_lock nunca bloqueia o scan; se ocupado, o próximo push salva.
    let _ = std::fs::write(p, serde_json::to_string(h).unwrap_or_default());
    // Espelha no volume Docker quando rodando com /data.
    if p != "/data/history.json" && std::path::Path::new("/data").exists() {
        let _ = std::fs::write("/data/history.json", serde_json::to_string(h).unwrap_or_default());
    }
}

impl AppState {
    fn new() -> Arc<Self> {
        let cfg = AppConfig::from_env();
        let proxies = load_proxies(&cfg.proxies_file);
        let watchlist = load_watchlist(&cfg.watchlist_file);
        let history = load_history(&history_file());
        let discord_tokens = load_discord_tokens();
        let tiktok_sessions = load_tiktok_sessions();
        tracing::info!(
            "l96a1-api proxies={} file={} timeout={}s conc={} watchlist={} history={}",
            proxies.len(),
            cfg.proxies_file,
            cfg.timeout_secs,
            cfg.max_concurrent,
            watchlist.len(),
            history.len(),
        );
        tracing::info!(
            "l96a1-api discord_tokens={} (0 = checker roda só unauthed, muro shared vem rápido)",
            discord_tokens.len(),
        );
        tracing::info!(
            "l96a1-api tiktok_sessions={} (0 = checker roda só unauthed oembed+page)",
            tiktok_sessions.len(),
        );
        tracing::info!(
            "l96a1-api state files: history={} checked={} dataset={}",
            history_file(),
            resolve_state_file("CHECKED_FILE", "checked.json"),
            resolve_dataset_dir(&cfg.dataset_dir),
        );
        Arc::new(Self {
            rotator: Arc::new(l96a1_proxy::Rotator::new(proxies)),
            discord_tokens,
            tiktok_sessions,
            version: env!("CARGO_PKG_VERSION").to_string(),
            snipe: Mutex::new(SnipeStatus {
                watchlist_size: watchlist.len(),
                ..Default::default()
            }),
            cfg,
            jobs: Mutex::new(HashMap::new()),
            history: Mutex::new(history),
            watchlist: Mutex::new(watchlist),
            bot_setup: Mutex::new(None),
            enrich_cache: enrich::EnrichCache::new(),
            checked_cache: Arc::new(checked::CheckedCache::new(
                std::env::var("CHECKED_TTL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(86400),
                resolve_state_file("CHECKED_FILE", "checked.json"),
            )),
            cancel_flags: Mutex::new(HashMap::new()),
        })
    }
}

/// Candidatos de arquivo independente do CWD: configurado, /data (Docker),
/// raiz do repo (cargo run de qualquer pasta), ao lado do binário.
/// Substitui os `C:\Dev\...` hardcoded (quebravam em outra máquina).
fn file_candidates(configured: &str, name: &str) -> Vec<String> {
    let mut c = vec![configured.to_string(), format!("/data/{name}")];
    c.push(format!("{}/../../{name}", env!("CARGO_MANIFEST_DIR")));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe.parent() {
            c.push(p.join(name).to_string_lossy().to_string());
        }
    }
    c
}

fn load_watchlist(configured: &str) -> Vec<String> {
    // 1 nick por linha, `#` comenta — mesmo formato do `l9 snipe`.
    for cand in file_candidates(configured, "watchlist.txt") {
        if let Ok(text) = std::fs::read_to_string(&cand) {
            let mut v: Vec<String> = text
                .lines()
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            v.sort();
            if !v.is_empty() || std::path::Path::new(&cand).exists() {
                return v;
            }
        }
    }
    Vec::new()
}

fn save_watchlist(configured: &str, users: &[String]) {
    // Persiste no arquivo configurado quando ele é gravável (local);
    // no Docker (/data) o volume cobre. Falha silenciosa = memória vale.
    let mut t = String::new();
    for u in users {
        t.push_str(u);
        t.push('\n');
    }
    let _ = std::fs::write(configured, t);
}

/// POST Discord webhook `{content}`. Retorna Ok(false) se sem URL.
async fn notify_webhook(url: &str, content: String) -> bool {
    if url.trim().is_empty() {
        return false;
    }
    let client = l96a1_core::build_client(Duration::from_secs(10), None);
    match client
        .post(url)
        .json(&serde_json::json!({"content": content}))
        .send()
        .await
    {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}

fn load_proxies(configured: &str) -> Vec<String> {
    for cand in file_candidates(configured, "proxies.txt") {
        let urls = l96a1_proxy::load_file(&cand);
        if !urls.is_empty() {
            return urls;
        }
        // Arquivo existe mas vazio continua procurando? Não — usa o configurado.
        if std::path::Path::new(&cand).exists() {
            return urls;
        }
    }
    Vec::new()
}

/// Sessões TikTok logadas (uma cookie-string por conta, espelho do
/// `discord_tokens.txt`). Fontes: env `TIKTOK_SESSION`/`TIKTOK_SESSIONS`
/// (sessões separadas por linha — cookies contêm `;`, então vírgula/ponto
/// e vírgula NÃO servem de separador) + arquivo `tiktok_sessions.txt`.
/// Como extrair: DevTools (F12) na conta logada → Application → Cookies →
/// copiar os pares `nome=valor` do domínio tiktok.com numa linha.
/// Nunca commitar (ver `.gitignore`).
fn load_tiktok_sessions() -> Vec<String> {
    let mut out = Vec::new();
    for env_key in ["TIKTOK_SESSION", "TIKTOK_SESSIONS"] {
        if let Ok(v) = std::env::var(env_key) {
            for sess in v.split('\n') {
                let t = sess.trim();
                if !t.is_empty() && !t.starts_with('#') && !out.iter().any(|x| x == t) {
                    out.push(t.to_string());
                }
            }
        }
    }
    for cand in file_candidates("tiktok_sessions.txt", "tiktok_sessions.txt") {
        if let Ok(text) = std::fs::read_to_string(&cand) {
            for l in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
                if !out.iter().any(|x| x == l) {
                    out.push(l.to_string());
                }
            }
        }
    }
    out
}

fn load_discord_tokens() -> Vec<String> {
    let mut out = Vec::new();
    for env_key in ["DISCORD_TOKEN", "DISCORD_TOKENS"] {
        if let Ok(v) = std::env::var(env_key) {
            for tok in v.split([',', '\n', ';']) {
                let t = tok.trim();
                if !t.is_empty() && !out.iter().any(|x| x == t) {
                    out.push(t.to_string());
                }
            }
        }
    }
    for cand in file_candidates("discord_tokens.txt", "discord_tokens.txt") {
        if let Ok(text) = std::fs::read_to_string(&cand) {
            for l in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
                if !out.iter().any(|x| x == l) {
                    out.push(l.to_string());
                }
            }
        }
    }
    out
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn new_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() % 1_000_000)
        .unwrap_or(0);
    format!("{prefix}_{secs}{nanos:09}")
}

// ---------------------------------------------------------------- health

async fn health(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "ok": true,
        "service": "l96a1-api",
        "version": st.version,
        "proxies": st.rotator.len(),
        "proxies_alive": st.rotator.alive_count(),
    }))
}

// ------------------------------------------------------------- platforms

async fn platforms() -> impl IntoResponse {
    let list: Vec<PlatformInfo> = l96a1_checkers::all()
        .iter()
        .map(|p| PlatformInfo {
            key: p.key().to_string(),
            name: p.name().to_string(),
            group: p.group().to_string(),
            hasChecker: true,
            max_concurrent: p.profile().max_concurrent,
            route: format!("/v1/{}/{{handle}}", p.key()),
        })
        .collect();
    Json(serde_json::json!({
        "total": list.len(),
        "platforms": list,
        "groups": l96a1_checkers::GROUPS,
        "note": "registro unificado: 29 hand-written + geradas mf:/wm: (tools/gen_sites.py)",
    }))
}

async fn catalogs() -> impl IntoResponse {
    // Catálogos JSON viraram fonte do gerador (tools/gen_sites.py); o motor
    // só conhece structs: total = all().len().
    let total = l96a1_checkers::all().len();
    let generated = l96a1_checkers::all_generated().len();
    Json(serde_json::json!({
        "handwritten": total - generated,
        "generated": generated,
        "total_indexed": total,
    }))
}

// ------------------------------------------------------------------ SSE

#[derive(Debug, serde::Deserialize)]
struct SearchQuery {
    q: Option<String>,
    query: Option<String>,
    platforms: Option<String>,
}

fn sse_event(name: &str, data: String) -> Event {
    Event::default().event(name).data(data)
}

/// GET /v1/search?q=adoecido&platforms=all — stream real do motor.
async fn search_sse(
    State(st): State<Arc<AppState>>,
    Query(qs): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = qs
        .q
        .or(qs.query)
        .unwrap_or_default()
        .trim()
        .to_string();
    if q.is_empty() || q.len() > 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "q vazio ou >64 chars"})),
        )
            .into_response();
    }
    let platforms_q = qs.platforms.unwrap_or_else(|| "all".to_string());
    let plats = resolve_platforms(&platforms_q);
    if plats.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "nenhuma plataforma bate"})),
        )
            .into_response();
    }
    let total = plats.len();
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(256);
    let st2 = st.clone();
    let q2 = q.clone();
    tokio::spawn(async move {
        let _ = tx
            .send(Ok(sse_event(
                "connected",
                r#"{"status":"connected"}"#.into(),
            )))
            .await;
        // (Sem evento `ratelimit` fake: o motor não impõe cota — quota real
        // vive no BFF website. Audit 2026-10 removeu limit:10 fixo.)
        let _ = tx
            .send(Ok(sse_event(
                "start",
                serde_json::json!({"total": total}).to_string(),
            )))
            .await;
        let cfg = st2.cfg.engine();
        let checked_n = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let checked_tx = tx.clone();
        let hits_n = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let hits_tx = hits_n.clone();
        // Plataformas onde o nick está LIVRE (cap 200 p/ não inundar).
        let frees = Arc::new(Mutex::new(Vec::<String>::new()));
        let frees_tx = frees.clone();
        let (hits, frees_n, checked, _warns, rates, errors, unknowns) = run_scan(
            vec![q2.clone()],
            plats,
            st2.rotator.clone(),
            st2.discord_tokens.clone(),
            st2.tiktok_sessions.clone(),
            cfg,
            {
                let tx = tx.clone();
                move |r: SearchResult| {
                    let hits_tx = hits_tx.clone();
                    hits_tx.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let data = serde_json::to_string(&r).unwrap_or_default();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(sse_event("result", data))).await;
                    });
                }
            },
            {
                let tx = tx.clone();
                let frees_tx = frees_tx.clone();
                move |f: models::FreeNick| {
                    let tx = tx.clone();
                    let frees_tx = frees_tx.clone();
                    tokio::spawn(async move {
                        let mut v = frees_tx.lock().await;
                        if v.len() < 200 && !v.contains(&f.platform) {
                            v.push(f.platform.clone());
                        }
                        drop(v);
                        let data = serde_json::to_string(&f).unwrap_or_default();
                        let _ = tx.send(Ok(sse_event("free", data))).await;
                    });
                }
            },
            {
                let tx = tx.clone();
                move |w: models::WarnInfo| {
                    let data = serde_json::to_string(&w).unwrap_or_default();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(sse_event("warn", data))).await;
                    });
                }
            },
            {
                let checked_n = checked_n.clone();
                move |n: usize| {
                    checked_n.store(n, std::sync::atomic::Ordering::SeqCst);
                    let tx = checked_tx.clone();
                    let data = serde_json::json!({"total": n}).to_string();
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(sse_event("checked", data))).await;
                    });
                }
            },
            Arc::new(AtomicBool::new(false)),
            st2.checked_cache.clone(),
        )
        .await;
        // Pequena espera para os sends spawnados chegarem antes do done.
        tokio::time::sleep(Duration::from_millis(150)).await;
        let _ = tx
            .send(Ok(sse_event(
                "done",
                serde_json::json!({"type": "done", "total": total, "hits": hits, "frees": frees_n, "checked": checked, "rates": rates, "errors": errors, "unknowns": unknowns})
                    .to_string(),
            )))
            .await;
        let free_list = frees.lock().await.clone();
        // Cliente desconectou no meio (Parar) = cancelado; senão, completed.
        let status = if tx.is_closed() { "cancelled" } else { "completed" };
        push_history(&st2, &q2, "username", free_list, status).await;
        st2.checked_cache.save();
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

/// Uma linha por scan: `available` = SÓ disponíveis (nicks livres no
/// bulk/dataset, plataformas livres no single). Hits ficam no stream.
/// `status` = completed | cancelled (Parar no meio registra cancelado).
/// Persistido em `history.json` (HISTORY_FILE) — não reseta no restart.
async fn push_history(st: &Arc<AppState>, query: &str, qtype: &str, available: Vec<String>, status: &str) {
    let snapshot = {
        let mut h = st.history.lock().await;
        h.push(HistoryEntry {
            query: query.to_string(),
            queryType: qtype.to_string(),
            resultCount: available.len(),
            status: status.to_string(),
            time: "just now".to_string(),
            created_at: now_iso(),
            available,
        });
        if h.len() > 200 {
            let drain = h.len() - 200;
            h.drain(0..drain);
        }
        h.clone()
    };
    save_history(&history_file(), &snapshot);
}

// ------------------------------------------------------------- bulk SSE

#[derive(Debug, serde::Deserialize)]
struct BulkQuery {
    users: Option<String>,
    q: Option<String>,
    platforms: Option<String>,
}

/// GET /v1/bulk?users=a,b,c&platforms=all — multi-nick direto via SSE.
/// Aceita `users=a,b,c` ou `users=a\nb\nc` (URL-encoded) ou `q=` single.
async fn bulk_sse(State(st): State<Arc<AppState>>, Query(qs): Query<BulkQuery>) -> impl IntoResponse {
    let raw = qs.users.or(qs.q).unwrap_or_default();
    // Dedupe aqui também (frontend já dedupa): mesmo user nunca 2x.
    let users: Vec<String> = dedupe_usernames(
        raw.split([',', '\n', ';']).map(|s| s.to_string()).collect(),
    )
    .into_iter()
    .take(100)
    .collect();
    if users.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "users vazio (max 100/job)"})),
        )
            .into_response();
    }
    let platforms_q = qs.platforms.unwrap_or_else(|| "all".to_string());
    let plats = resolve_platforms(&platforms_q);
    if plats.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "nenhuma plataforma bate"})),
        )
            .into_response();
    }
    let total = users.len() * plats.len();
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(512);
    let st2 = st.clone();
    tokio::spawn(async move {
        let _ = tx
            .send(Ok(sse_event(
                "connected",
                r#"{"status":"connected"}"#.into(),
            )))
            .await;
        let _ = tx
            .send(Ok(sse_event(
                "start",
                serde_json::json!({"total": total, "usernames": users.len()}).to_string(),
            )))
            .await;
        let cfg = st2.cfg.engine();
        // Nicks livres (únicos, cap 500) — 1 linha de histórico no final.
        let free_nicks = Arc::new(Mutex::new(Vec::<String>::new()));
        let free_nicks_tx = free_nicks.clone();
        let (hits, frees_n, checked, _warns, rates, errors, unknowns) = run_scan(
            users.clone(),
            plats,
            st2.rotator.clone(),
            st2.discord_tokens.clone(),
            st2.tiktok_sessions.clone(),
            cfg,
            {
                let tx = tx.clone();
                move |r: SearchResult| {
                    let data = serde_json::to_string(&r).unwrap_or_default();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(sse_event("result", data))).await;
                    });
                }
            },
            {
                let tx = tx.clone();
                let free_nicks_tx = free_nicks_tx.clone();
                move |f: models::FreeNick| {
                    let tx = tx.clone();
                    let free_nicks_tx = free_nicks_tx.clone();
                    tokio::spawn(async move {
                        {
                            let mut v = free_nicks_tx.lock().await;
                            if v.len() < 500 && !v.contains(&f.nick) {
                                v.push(f.nick.clone());
                            }
                        }
                        let data = serde_json::to_string(&f).unwrap_or_default();
                        let _ = tx.send(Ok(sse_event("free", data))).await;
                    });
                }
            },
            {
                let tx = tx.clone();
                move |w: models::WarnInfo| {
                    let data = serde_json::to_string(&w).unwrap_or_default();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(sse_event("warn", data))).await;
                    });
                }
            },
            {
                let tx = tx.clone();
                move |n: usize| {
                    let tx = tx.clone();
                    let data = serde_json::json!({"total": n}).to_string();
                    tokio::spawn(async move {
                        let _ = tx.send(Ok(sse_event("checked", data))).await;
                    });
                }
            },
            Arc::new(AtomicBool::new(false)),
            st2.checked_cache.clone(),
        )
        .await;
        tokio::time::sleep(Duration::from_millis(150)).await;
        let _ = tx
            .send(Ok(sse_event(
                "done",
                serde_json::json!({"type":"done","total": total, "hits": hits, "frees": frees_n, "checked": checked, "rates": rates, "errors": errors, "unknowns": unknowns})
                    .to_string(),
            )))
            .await;
        // COMPACTO: 1 linha por scan bulk (nicks livres). Parar no meio = cancelled.
        let label = format!("bulk · {} nicks", users.len());
        let available = free_nicks.lock().await.clone();
        let status = if tx.is_closed() { "cancelled" } else { "completed" };
        push_history(&st2, &label, "bulk", available, status).await;
        st2.checked_cache.save();
    });
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

// ------------------------------------------------------------ scan jobs

#[derive(Debug, serde::Deserialize)]
struct CreateScan {
    usernames: Option<Vec<String>>,
    users: Option<String>,
    platforms: Option<String>,
    source: Option<String>,
}

async fn create_scan(
    State(st): State<Arc<AppState>>,
    Json(body): Json<CreateScan>,
) -> impl IntoResponse {
    let mut usernames: Vec<String> = body.usernames.unwrap_or_default();
    if let Some(u) = body.users {
        usernames.extend(
            u.split([',', '\n', ';'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
    }
    // Dedupe global do job: mesmo user nunca 2x na mesma passada.
    usernames = dedupe_usernames(usernames).into_iter().take(500).collect();
    if usernames.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "usernames vazio (max 500/job)"})),
        )
            .into_response();
    }
    let platforms_q = body.platforms.unwrap_or_else(|| "all".to_string());
    let plats = resolve_platforms(&platforms_q);
    if plats.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "nenhuma plataforma bate"})),
        )
            .into_response();
    }
    let id = spawn_scan_job(
        &st,
        usernames,
        plats,
        platforms_q,
        body.source.unwrap_or_else(|| "bulk".to_string()),
        100,
    )
    .await;
    let total = st
        .jobs
        .lock()
        .await
        .get(&id)
        .map(|j| j.total)
        .unwrap_or(0);
    (StatusCode::ACCEPTED, Json(serde_json::json!({"jobId": id, "total": total}))).into_response()
}

/// Cria o job e dispara o scan em background. Retorna o id.
/// `batch` = tamanho do lote (1..=100); a fila anda de batch em batch
/// até o fim no MESMO job. Usado por /v1/scans, /v1/dataset/scan e testes.
async fn spawn_scan_job(
    st: &Arc<AppState>,
    usernames: Vec<String>,
    plats: Vec<Arc<dyn l96a1_core::Platform + 'static>>,
    platforms_q: String,
    source: String,
    batch: usize,
) -> String {
    let id = new_id("scan");
    let total = usernames.len() * plats.len();
    let source_label = source.clone();
    let job = ScanJob {
        id: id.clone(),
        status: JobStatus::Running,
        source,
        usernames: usernames.clone(),
        platforms: platforms_q,
        total,
        checked: 0,
        hits: 0,
        created_at: now_iso(),
        results: Vec::new(),
        available: Vec::new(),
        error: None,
        warnings: Vec::new(),
    };
    st.jobs.lock().await.insert(id.clone(), job);
    let flag = Arc::new(AtomicBool::new(false));
    st.cancel_flags.lock().await.insert(id.clone(), flag.clone());
    let st2 = st.clone();
    let job_id = id.clone();
    let nick_count = usernames.len();
    tokio::spawn(async move {
        let cfg = st2.cfg.engine();
        // Fila por lotes: checa de batch em batch até o final no MESMO job
        // (1 linha de histórico no fim). Batch máximo = 100.
        let size = batch.clamp(1, 100);
        let chunks: Vec<Vec<String>> = usernames
            .chunks(size)
            .map(|c| c.to_vec())
            .collect();
        let mut checked_total = 0usize;
        for chunk in chunks {
            if flag.load(Ordering::Relaxed) {
                break;
            }
            let base = checked_total;
            let st_hit = st2.clone();
            let jid_hit = job_id.clone();
            let st_free = st2.clone();
            let jid_free = job_id.clone();
            let st_warn = st2.clone();
            let jid_warn = job_id.clone();
            let st_chk = st2.clone();
            let jid_chk = job_id.clone();
            let (_h, _f, checked, _w, _r, _e, _u) = run_scan(
                chunk,
                plats.clone(),
                st2.rotator.clone(),
                st2.discord_tokens.clone(),
                st2.tiktok_sessions.clone(),
                cfg.clone(),
                {
                    let st2 = st_hit.clone();
                    let job_id = jid_hit.clone();
                    move |r: SearchResult| {
                        let st2 = st2.clone();
                        let job_id = job_id.clone();
                        tokio::spawn(async move {
                            let mut jobs = st2.jobs.lock().await;
                            if let Some(j) = jobs.get_mut(&job_id) {
                                j.results.push(r);
                                j.hits = j.results.len();
                            }
                        });
                    }
                },
                {
                    let st2 = st_free.clone();
                    let job_id = jid_free.clone();
                    move |f: models::FreeNick| {
                        let st2 = st2.clone();
                        let job_id = job_id.clone();
                        tokio::spawn(async move {
                            let mut jobs = st2.jobs.lock().await;
                            if let Some(j) = jobs.get_mut(&job_id) {
                                if j.available.len() < 2000 {
                                    j.available.push(f);
                                }
                            }
                        });
                    }
                },
                {
                    let st2 = st_warn.clone();
                    let job_id = jid_warn.clone();
                    move |w: models::WarnInfo| {
                        let st2 = st2.clone();
                        let job_id = job_id.clone();
                        tokio::spawn(async move {
                            let cap = st2.cfg.warn_cap;
                            let mut jobs = st2.jobs.lock().await;
                            if let Some(j) = jobs.get_mut(&job_id) {
                                if j.warnings.len() < cap {
                                    j.warnings.push(w);
                                }
                            }
                        });
                    }
                },
                {
                    let st2 = st_chk.clone();
                    let job_id = jid_chk.clone();
                    move |n: usize| {
                        let st2 = st2.clone();
                        let job_id = job_id.clone();
                        tokio::spawn(async move {
                            let mut jobs = st2.jobs.lock().await;
                            if let Some(j) = jobs.get_mut(&job_id) {
                                j.checked = base + n;
                            }
                        });
                    }
                },
                flag.clone(),
                st2.checked_cache.clone(),
            )
            .await;
            checked_total += checked;
            // Sincroniza o contador (os pushes spawnados chegam fora de ordem).
            tokio::time::sleep(Duration::from_millis(150)).await;
            {
                let mut jobs = st2.jobs.lock().await;
                if let Some(j) = jobs.get_mut(&job_id) {
                    j.checked = checked_total;
                    j.hits = j.results.len();
                }
            }
        }
        let was_cancelled = flag.load(Ordering::Relaxed);
        st2.cancel_flags.lock().await.remove(&job_id);
        // Flush final pode ter chegado fora de ordem; consolida contadores.
        let jobs = st2.jobs.lock().await;
        if jobs.get(&job_id).is_some() {
            drop(jobs);
            tokio::time::sleep(Duration::from_millis(300)).await;
            let mut jobs = st2.jobs.lock().await;
            if let Some(j) = jobs.get_mut(&job_id) {
                // Se cancelaram DURANTE o flush, mantém Cancelled.
                if flag.load(Ordering::Relaxed) {
                    j.status = JobStatus::Cancelled;
                } else {
                    j.checked = checked_total;
                    j.hits = j.results.len();
                    j.status = JobStatus::Done;
                }
            }
        }
        // 1 linha de histórico por job (só disponíveis + o que já checou).
        // Cancelado também entra, com status cancelled.
        let status = if was_cancelled { "cancelled" } else { "completed" };
        let free_nicks: Vec<String> = {
            let jobs = st2.jobs.lock().await;
            let mut seen = std::collections::HashSet::new();
            jobs.get(&job_id)
                .map(|j| {
                    j.available
                        .iter()
                        .map(|f| f.nick.clone())
                        .filter(|n| seen.insert(n.clone()))
                        .take(500)
                        .collect()
                })
                .unwrap_or_default()
        };
        let label = format!("{source_label} · {nick_count} nicks");
        push_history(&st2, &label, &source_label, free_nicks, status).await;
        st2.checked_cache.save();
    });
    id
}

async fn get_scan(State(st): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    let jobs = st.jobs.lock().await;
    match jobs.get(&id) {
        Some(j) => Json(serde_json::to_value(j).unwrap()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "job não encontrado"})),
        )
            .into_response(),
    }
}

async fn list_scans(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let jobs = st.jobs.lock().await;
    let mut v: Vec<&ScanJob> = jobs.values().collect();
    v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let out: Vec<&ScanJob> = v.into_iter().take(50).collect();
    Json(serde_json::json!({"scans": out, "total": jobs.len()})).into_response()
}

/// POST /v1/scans/:id/cancel — para o job entre chunks (fila 100/100).
/// O resultado parcial vai para o history com status cancelled + livres.
async fn cancel_scan(State(st): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    let running = matches!(
        st.jobs.lock().await.get(&id).map(|j| j.status.clone()),
        Some(JobStatus::Running)
    );
    if !running {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "job não está rodando"})),
        )
            .into_response();
    }
    if let Some(flag) = st.cancel_flags.lock().await.get(&id) {
        flag.store(true, Ordering::Relaxed);
    }
    // Marca já p/ o polling ver na hora; a task confirma no flush.
    {
        let mut jobs = st.jobs.lock().await;
        if let Some(j) = jobs.get_mut(&id) {
            j.status = JobStatus::Cancelled;
        }
    }
    Json(serde_json::json!({"jobId": id, "status": "cancelled"})).into_response()
}

// --------------------------------------------------------------- checked
// Cache de já-checados (não-livres): evita re-check na rede.

async fn checked_stats(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "size": st.checked_cache.len().await,
        "ttl_secs": st.checked_cache.ttl_secs().await,
    }))
    .into_response()
}

async fn checked_clear(
    State(st): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(nick) = q.get("nick").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let n = st.checked_cache.clear_nick(nick).await;
        return Json(serde_json::json!({"cleared": n, "nick": nick})).into_response();
    }
    let n = st.checked_cache.clear().await;
    Json(serde_json::json!({"cleared": n})).into_response()
}

#[derive(Debug, serde::Deserialize)]
struct CheckedTtl {
    ttl_secs: Option<u64>,
}

async fn checked_ttl(
    State(st): State<Arc<AppState>>,
    Json(body): Json<CheckedTtl>,
) -> impl IntoResponse {
    let secs = body.ttl_secs.unwrap_or(86400);
    let v = st.checked_cache.set_ttl(secs).await;
    // Persiste a escolha? TTL vive em memória + env no boot; o valor atual
    // vale até reiniciar (suficiente p/ minutos/horas/dias da sessão).
    Json(serde_json::json!({"ttl_secs": v})).into_response()
}

// --------------------------------------------------------------- dataset

/// Resolve o diretório do dataset independente do CWD do processo:
/// 1) o configurado (DATASET_DIR), 2) `dataset/` da raiz do repo (via
/// CARGO_MANIFEST_DIR — funciona com `cargo run` de qualquer pasta),
/// 3) ao lado do binário, 4) `/data/dataset` (Docker).
/// Prefere o primeiro que rende arquivos; senão o primeiro que existe.
fn resolve_dataset_dir(configured: &str) -> String {
    let mut cands = vec![configured.to_string()];
    cands.push(concat!(env!("CARGO_MANIFEST_DIR"), "/../../dataset").to_string());
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe.parent() {
            cands.push(p.join("dataset").to_string_lossy().to_string());
        }
    }
    cands.push("/data/dataset".to_string());
    for c in &cands {
        if !l96a1_dataset::list_files(c).is_empty() {
            return c.clone();
        }
    }
    for c in &cands {
        if std::path::Path::new(c).is_dir() {
            return c.clone();
        }
    }
    configured.to_string()
}

async fn dataset_files(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let dir = resolve_dataset_dir(&st.cfg.dataset_dir);
    let mut files = l96a1_dataset::list_files(&dir);
    // Sem cap baixo: o dataset tem 2700+ .txt (antes cortava em 500 e
    // pastas inteiras sumiam da lista). Cap alto só anti-abuso.
    files = files.into_iter().take(6000).collect();
    let out: Vec<serde_json::Value> = files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f,
                "count": l96a1_dataset::count_file(f),
            })
        })
        .collect();
    Json(serde_json::json!({"dataset_dir": dir, "files": out, "total_files": out.len()}))
        .into_response()
}

#[derive(Debug, serde::Deserialize)]
struct DatasetScan {
    files: Option<Vec<String>>,
    /// Filtro substring sobre path/bucket (ex: "3C", "forebears", "patterns").
    /// Vazio + sem files = todos os arquivos (cap 500).
    filter: Option<String>,
    smart: Option<bool>,
    limit: Option<usize>,
    platforms: Option<String>,
}

/// POST /v1/dataset/scan — monta candidatos via l96a1-dataset
/// (smart scoring + boost Forebears/Trends) e cria job.
/// `limit` = tamanho do lote (1..=100); o job percorre TODOS os candidatos
/// de batch em batch até o final (cap de segurança: 5000 nicks/job).
async fn dataset_scan(
    State(st): State<Arc<AppState>>,
    Json(body): Json<DatasetScan>,
) -> impl IntoResponse {
    let dir = resolve_dataset_dir(&st.cfg.dataset_dir);
    let mut files = l96a1_dataset::list_files(&dir);
    if let Some(want) = body.files.filter(|v| !v.is_empty()) {
        let set: std::collections::HashSet<String> = want.into_iter().collect();
        files.retain(|f| set.contains(f));
    } else if let Some(f) = body.filter.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()) {
        files.retain(|p| p.to_lowercase().contains(&f));
    }
    files = files.into_iter().take(6000).collect();
    if files.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "nenhum arquivo bate (filter/files)"})),
        )
            .into_response();
    }
    let smart = body.smart.unwrap_or(true);
    // `limit` aqui é o tamanho do lote (máx 100); o total percorrido é
    // todos os candidatos dos arquivos (cap anti-abuso: 5000/job).
    let batch = body.limit.unwrap_or(100).clamp(1, 100);
    let pop = l96a1_dataset::load_popularity(&dir);
    let cands = l96a1_dataset::load_candidates(&files, &[], smart, &|n| pop.coveted(n));
    if cands.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "0 candidatos nos arquivos"})),
        )
            .into_response();
    }
    let usernames: Vec<String> = cands.into_iter().take(5000).map(|c| c.name).collect();
    let platforms_q = body.platforms.unwrap_or_else(|| "all".to_string());
    let plats = resolve_platforms(&platforms_q);
    if plats.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "nenhuma plataforma bate"})),
        )
            .into_response();
    }
    let id = spawn_scan_job(&st, usernames.clone(), plats, platforms_q, "dataset".to_string(), batch).await;
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({"jobId": id, "usernames": usernames.len(), "batch": batch})),
    )
        .into_response()
}

// --------------------------------------------------------------- proxies

async fn proxies_list(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "total": st.rotator.len(),
        "alive": st.rotator.alive_count(),
        "file": st.cfg.proxies_file,
        "note": "EMA/cooldown ao vivo no Rotator; POST /v1/proxies/test roda health-check",
    }))
    .into_response()
}

/// Alvos reais do health-check (gstatic = conectividade; demais = utilidade
/// para o username checker — um proxy pode passar num e falhar noutro).
const HEALTH_TARGETS: &[&str] = &[
    "http://gstatic.com/generate_204",
    "https://discord.com/",
    "https://github.com/",
];

#[derive(Debug, serde::Deserialize)]
struct ProxiesTestBody {
    targets: Option<Vec<String>>,
}

async fn proxies_test(
    State(st): State<Arc<AppState>>,
    body: Option<Json<ProxiesTestBody>>,
) -> impl IntoResponse {
    let urls = load_proxies(&st.cfg.proxies_file);
    if urls.is_empty() {
        return Json(serde_json::json!({"total": 0, "ranked": [], "note": "sem proxies — rota direta ativa"})).into_response();
    }
    // Alvos escolhidos na UI; default = todos os reais. Body opcional
    // (POST vazio testa tudo — compat com chamadas antigas).
    let targets: Vec<String> = body
        .and_then(|b| b.0.targets)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| HEALTH_TARGETS.iter().map(|s| s.to_string()).collect());
    let target_refs: Vec<&str> = targets.iter().map(String::as_str).collect();
    // TODOS os proxies (antes cortava nos 100 primeiros).
    let mut entries: Vec<l96a1_proxy::ProxyEntry> =
        urls.into_iter().map(l96a1_proxy::ProxyEntry::new).collect();
    let verdicts = l96a1_proxy::health_check_targets(&mut entries, &target_refs, 20).await;
    let verdict_of: HashMap<String, (usize, usize)> = verdicts
        .into_iter()
        .map(|(u, v)| (u, (v.ok_targets, v.targets)))
        .collect();
    // Funde no Rotator vivo: EMA/fails passam a valer de verdade.
    let mut live = 0usize;
    for e in &entries {
        let ok = verdict_of.get(&e.url).map(|(n, _)| *n > 0).unwrap_or(false);
        st.rotator.report(&e.url, ok, e.latency_ema_ms);
        if ok {
            live += 1;
        }
    }
    entries.sort_by(|a, b| {
        a.latency_ema_ms
            .partial_cmp(&b.latency_ema_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let ranked: Vec<serde_json::Value> = entries
        .into_iter()
        .take(100)
        .map(|e| {
            let (ok_t, total_t) =
                verdict_of.get(&e.url).copied().unwrap_or((0, target_refs.len()));
            serde_json::json!({
                "url": mask_proxy(&e.url),
                "protocol": e.protocol,
                "latency_ms": e.latency_ema_ms.round(),
                "success": e.success,
                "fails": e.fails,
                "healthy": e.healthy(),
                "targets_ok": format!("{ok_t}/{total_t}"),
                "dead": ok_t == 0,
            })
        })
        .collect();
    Json(serde_json::json!({
        "total": ranked.len(),
        "tested": st.rotator.len(),
        "live": live,
        "bad": ranked.len() - live.min(ranked.len()),
        "targets": targets,
        "ranked": ranked,
    }))
    .into_response()
}

/// POST /v1/proxies/prune — remove do pool vivo + `proxies.txt` (com `.bak`)
/// SÓ podres (`should_remove`: 8+ fails sem sucesso, ou 12+ fails com taxa
/// ruim). Flaky (passou >=1 alvo com taxa ok) nunca sai.
async fn proxies_prune(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let dead: Vec<String> = st
        .rotator
        .snapshot()
        .into_iter()
        .filter(|e| e.should_remove())
        .map(|e| e.url)
        .collect();
    if dead.is_empty() {
        return Json(serde_json::json!({"removed": 0, "note": "nada 100% morto (flaky preservados)"}))
            .into_response();
    }
    // Resolve o arquivo real (mesma fragilidade de CWD do dataset).
    let mut cands = vec![st.cfg.proxies_file.clone(), "/data/proxies.txt".to_string()];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe.parent() {
            cands.push(p.join("proxies.txt").to_string_lossy().to_string());
        }
    }
    let path = cands.into_iter().find(|c| std::path::Path::new(c).is_file());
    let Some(path) = path else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "proxies.txt não encontrado"})),
        )
            .into_response();
    };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    // Mesma normalização do load_file (linha sem esquema = http://).
    let norm = |t: &str| {
        if t.contains("://") {
            t.to_string()
        } else {
            format!("http://{t}")
        }
    };
    let kept: Vec<&str> = lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            t.is_empty() || t.starts_with('#') || !dead.iter().any(|d| *d == norm(t))
        })
        .map(|l| *l)
        .collect();
    // Backup + escrita (Docker :ro falha aqui com mensagem clara).
    let bak = format!("{path}.bak");
    let _ = std::fs::copy(&path, &bak);
    if let Err(e) = std::fs::write(&path, kept.join("\n")) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("arquivo somente-leitura? ({e}) — edite o proxies.txt local")}))
        )
            .into_response();
    }
    let removed = st.rotator.remove(&dead);
    Json(serde_json::json!({
        "removed": removed,
        "kept": kept.len(),
        "backup": bak,
        "note": "só 100% mortos; flaky preservados",
    }))
    .into_response()
}

fn mask_proxy(url: &str) -> String {
    l96a1_core::mask_proxy(Some(url))
}

/// GET /v1/proxies/unmask?url=<masked-ou-index> — revela integral (logado).
/// BFF exige sessão; aqui confiamos no BFF (rede interna). Retorna a URL
/// original apenas se ela constar no arquivo/proxy carregado.
async fn proxies_unmask(
    State(st): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let want = q.get("url").cloned().unwrap_or_default();
    let urls = load_proxies(&st.cfg.proxies_file);
    // Aceita máscara (`http://***:8080`) ou sufixo; nunca lista tudo.
    let hit = urls.into_iter().find(|u| {
        l96a1_core::mask_proxy(Some(u)) == want || u.ends_with(&want) || *u == want
    });
    match hit {
        Some(u) => Json(serde_json::json!({"url": u})).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "proxy não encontrado"}))).into_response(),
    }
}

// --------------------------------------------------------------- enrich
// API separada do check (Fase 1): só dado público, pool próprio, cache 24h.

#[derive(Debug, serde::Deserialize)]
struct EnrichQuery {
    platform: Option<String>,
    user: Option<String>,
    nick: Option<String>,
}

async fn enrich_one_route(
    State(st): State<Arc<AppState>>,
    Query(q): Query<EnrichQuery>,
) -> impl IntoResponse {
    let platform = q.platform.unwrap_or_default().trim().to_lowercase();
    let user = q.user.or(q.nick).unwrap_or_default().trim().to_string();
    if platform.is_empty() || user.is_empty() || user.len() > 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "use ?platform=<key>&user=<nick>"})),
        )
            .into_response();
    }
    let cache_key = format!("{platform}|{}", user.to_lowercase());
    if let Some(hit) = st.enrich_cache.inner_get(&cache_key).await {
        return Json(serde_json::json!({
            "platform": platform, "user": user, "cached": true, "profile": hit,
        }))
        .into_response();
    }
    let prof = enrich::enrich_one(
        &platform,
        &user,
        &st.cfg.github_token,
        st.discord_tokens.first().map(String::as_str),
    )
    .await;
    st.enrich_cache.inner_put(cache_key, prof.clone()).await;
    Json(serde_json::json!({"platform": platform, "user": user, "cached": false, "profile": prof})).into_response()
}

#[derive(Debug, serde::Deserialize)]
struct EnrichBatch {
    items: Vec<EnrichItem>,
}

#[derive(Debug, serde::Deserialize)]
struct EnrichItem {
    platform: String,
    user: String,
}

async fn enrich_batch_route(
    State(st): State<Arc<AppState>>,
    Json(body): Json<EnrichBatch>,
) -> impl IntoResponse {
    let mut out = Vec::new();
    for it in body.items.into_iter().take(20) {
        let platform = it.platform.trim().to_lowercase();
        let user = it.user.trim().to_string();
        if platform.is_empty() || user.is_empty() || user.len() > 64 {
            continue;
        }
        let cache_key = format!("{platform}|{}", user.to_lowercase());
        if let Some(hit) = st.enrich_cache.inner_get(&cache_key).await {
            out.push(serde_json::json!({"platform": platform, "user": user, "cached": true, "profile": hit}));
            continue;
        }
        let prof = enrich::enrich_one(
            &platform,
            &user,
            &st.cfg.github_token,
            st.discord_tokens.first().map(String::as_str),
        )
        .await;
        st.enrich_cache.inner_put(cache_key, prof.clone()).await;
        out.push(serde_json::json!({"platform": platform, "user": user, "cached": false, "profile": prof}));
    }
    Json(serde_json::json!({"results": out})).into_response()
}

// --------------------------------------------------------------- history

async fn history(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let h = st.history.lock().await;
    let mut v = h.clone();
    v.reverse();
    Json(serde_json::json!({"history": v.iter().take(50).collect::<Vec<_>>(), "total": v.len()})).into_response()
}

/// DELETE /v1/history[?query=] — apaga 1 linha ou tudo. Apagar libera
/// re-check (o frontend evoca o cache de checados junto).
/// Persiste o arquivo junto (senão o restart "ressuscita" linhas apagadas).
async fn history_delete(
    State(st): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let snapshot: Option<Vec<HistoryEntry>>;
    let resp;
    {
        let mut h = st.history.lock().await;
        if let Some(want) = q.get("query").map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let before = h.len();
            h.retain(|e| e.query != want);
            let removed = before - h.len();
            snapshot = Some(h.clone());
            resp = Json(serde_json::json!({"removed": removed, "query": want})).into_response();
        } else {
            let removed = h.len();
            h.clear();
            snapshot = Some(Vec::new());
            resp = Json(serde_json::json!({"removed": removed})).into_response();
        }
    }
    if let Some(s) = snapshot {
        save_history(&history_file(), &s);
    }
    resp
}

async fn rate_limit(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    // Honesto: o motor não impõe cota (números fixos 10/25 removidos no
    // audit 2026-10). Quota real vive no BFF website (/api/rate-limit).
    let used = st.history.lock().await.len();
    Json(serde_json::json!({
        "managed": false,
        "searches_total": used,
        "note": "motor sem cota; quota real no BFF website",
    }))
    .into_response()
}

// ---------------------------------------------------------------- doctor

async fn doctor(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let plats = l96a1_checkers::all();
    let router = l96a1_proxy::HybridRouter::default();
    let routes: Vec<serde_json::Value> = plats
        .iter()
        .map(|p| {
            serde_json::json!({
                "key": p.key(),
                "name": p.name(),
                "group": p.group(),
                "route": router.route_label(p.key(), &p.profile()),
                "concurrency": p.profile().max_concurrent,
            })
        })
        .collect();
    Json(serde_json::json!({
        "version": st.version,
        "timeout_secs": st.cfg.timeout_secs,
        "max_concurrent": st.cfg.max_concurrent,
        "proxies": {"total": st.rotator.len(), "alive": st.rotator.alive_count(), "file": st.cfg.proxies_file},
        "platforms": {"total": plats.len(), "routes": routes},
        "dataset_dir": st.cfg.dataset_dir,
        "github_token": !st.cfg.github_token.is_empty(),
        "discord_tokens": st.discord_tokens.len(),
        "discord_tokens_health": l96a1_checkers::discord::token_health(&st.discord_tokens),
        "tiktok_sessions": st.tiktok_sessions.len(),
        "tiktok_sessions_health": l96a1_checkers::tiktok_session::session_health(&st.tiktok_sessions),
    }))
    .into_response()
}

// ---------------------------------------------------------------- snipe
// Sniper 24h (equivale a `l9 snipe`): observa a watchlist em loop e
// notifica nicks LIBERADOS (Available) no webhook. Transições Taken→
// Available são o sinal; Unknown/RateLimited nunca notificam.

/// Um ciclo: probe em toda (nick x plataforma), retorna liberados.
async fn snipe_cycle(st: &Arc<AppState>) -> Vec<serde_json::Value> {
    let watch: Vec<String> = st.watchlist.lock().await.clone();
    if watch.is_empty() {
        return Vec::new();
    }
    let plats = resolve_platforms(&st.cfg.snipe_platforms);
    if plats.is_empty() {
        return Vec::new();
    }
    let cfg = st.cfg.engine();
    let router = l96a1_proxy::HybridRouter::default();
    let base = Arc::new(l96a1_core::build_client(
        Duration::from_secs(cfg.timeout_secs),
        None,
    ));
    let global_cap = cfg.max_concurrent.clamp(1, 300);
    let global_sem = Arc::new(tokio::sync::Semaphore::new(global_cap));
    // Pacing por plataforma (audit 2026-10): o sniper ignorava
    // `min_interval_ms` e se auto-429ava (discord 500ms, minecraft 300ms).
    let buckets = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::<String, std::time::Instant>::new(),
    ));
    let mut set = tokio::task::JoinSet::new();
    for nick in watch {
        for plat in plats.clone() {
            let global_sem = global_sem.clone();
            let base = base.clone();
            let rotator = st.rotator.clone();
            let router = router.clone();
            let tokens = st.discord_tokens.clone();
            let tiktok_sessions = st.tiktok_sessions.clone();
            let cfg = cfg.clone();
            let buckets = buckets.clone();
            let nick2 = nick.clone();
            set.spawn(async move {
                let _g = global_sem.acquire_owned().await.ok()?;
                let wait_ms = {
                    let mut b = buckets.lock().await;
                    let now = std::time::Instant::now();
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
                let probe =
                    probe_one(plat, &nick2, base, rotator, &router, &tokens, &tiktok_sessions, &cfg).await?;
                if probe.result == l96a1_core::CheckResult::Available {
                    Some(serde_json::json!({
                        "nick": nick2,
                        "platform": probe.platform_name,
                        "key": probe.platform_key,
                        "profileUrl": probe.profile_url,
                    }))
                } else {
                    None
                }
            });
        }
    }
    let mut freed = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(Some(v)) = res {
            freed.push(v);
        }
    }
    freed
}

async fn run_snipe_cycle_and_notify(st: &Arc<AppState>) {
    let freed = snipe_cycle(st).await;
    let stamp = now_iso();
    {
        let mut s = st.snipe.lock().await;
        s.running = false;
        s.last_run = Some(stamp);
        s.cycles += 1;
        s.watchlist_size = st.watchlist.lock().await.len();
        s.next_in_secs = st.cfg.snipe_interval_secs;
        if !freed.is_empty() {
            s.hits_total += freed.len() as u64;
            s.last_hits = freed
                .iter()
                .take(20)
                .map(|v| {
                    format!(
                        "{} @ {}",
                        v.get("nick").and_then(|n| n.as_str()).unwrap_or("?"),
                        v.get("platform").and_then(|p| p.as_str()).unwrap_or("?")
                    )
                })
                .collect();
        }
    }
    if freed.is_empty() || st.cfg.webhook_url.trim().is_empty() {
        return;
    }
    // Batch: agrupa até 5 liberados por mensagem (igual `webhook.mode=batch`).
    for chunk in freed.chunks(5) {
        let mut msg = String::from("🎯 **L96A1 sniper — liberados:**\n");
        for v in chunk {
            msg.push_str(&format!(
                "• `{}` — {} {}\n",
                v.get("nick").and_then(|n| n.as_str()).unwrap_or("?"),
                v.get("platform").and_then(|p| p.as_str()).unwrap_or("?"),
                v.get("profileUrl").and_then(|u| u.as_str()).unwrap_or(""),
            ));
        }
        notify_webhook(&st.cfg.webhook_url, msg).await;
    }
}

async fn snipe_status(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let s = st.snipe.lock().await.clone();
    Json(serde_json::json!({
        "status": s,
        "config": {
            "interval_secs": st.cfg.snipe_interval_secs,
            "platforms": st.cfg.snipe_platforms,
            "webhook_configured": !st.cfg.webhook_url.trim().is_empty(),
            "watchlist_file": st.cfg.watchlist_file,
        },
    }))
    .into_response()
}

async fn snipe_watchlist(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let w = st.watchlist.lock().await.clone();
    Json(serde_json::json!({"watchlist": w, "size": w.len()})).into_response()
}

#[derive(Debug, serde::Deserialize)]
struct WatchlistEdit {
    action: String,
    nick: String,
}

async fn snipe_watchlist_edit(
    State(st): State<Arc<AppState>>,
    Json(body): Json<WatchlistEdit>,
) -> impl IntoResponse {
    let nick = body.nick.trim().to_lowercase();
    if nick.is_empty() || nick.len() > 32 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "nick vazio ou >32 chars"})),
        )
            .into_response();
    }
    let mut w = st.watchlist.lock().await;
    match body.action.as_str() {
        "add" => {
            if !w.contains(&nick) {
                w.push(nick.clone());
                w.sort();
            }
        }
        "remove" => w.retain(|u| u != &nick),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "action: add|remove"})),
            )
                .into_response();
        }
    }
    let out = w.clone();
    save_watchlist(&st.cfg.watchlist_file, &out);
    st.snipe.lock().await.watchlist_size = out.len();
    Json(serde_json::json!({"watchlist": out, "size": out.len()})).into_response()
}

/// POST /v1/snipe/run-once — dispara 1 ciclo em background (202).
async fn snipe_run_once(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    {
        let mut s = st.snipe.lock().await;
        if s.running {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "ciclo já rodando"})),
            )
                .into_response();
        }
        s.running = true;
    }
    let st2 = st.clone();
    tokio::spawn(async move {
        run_snipe_cycle_and_notify(&st2).await;
    });
    (StatusCode::ACCEPTED, Json(serde_json::json!({"started": true}))).into_response()
}

// -------------------------------------------------------------- webhooks

async fn webhooks(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "url_configured": !st.cfg.webhook_url.trim().is_empty(),
        "mode": st.cfg.webhook_mode,
    }))
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
struct WebhookTest {
    text: Option<String>,
}

async fn webhooks_test(
    State(st): State<Arc<AppState>>,
    Json(body): Json<WebhookTest>,
) -> impl IntoResponse {
    let text = body
        .text
        .unwrap_or_else(|| "⌖ L96A1 webhook online (teste via web).".to_string());
    let sent = notify_webhook(&st.cfg.webhook_url, text).await;
    if sent {
        Json(serde_json::json!({"sent": true})).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"sent": false, "error": "WEBHOOK_URL ausente ou entrega falhou"})),
        )
            .into_response()
    }
}

// ------------------------------------------------------------------- bot
// Bot Discord via web: status + setup do servidor (cargos, canais por
// plataforma, #verify/#hits/#log + webhooks). O gateway (poise 0.7,
// híbrido prefixo+slash) roda como sidecar 24h.

async fn bot_status(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let setup = st.bot_setup.lock().await.clone();
    Json(serde_json::json!({
        "token_configured": !st.cfg.bot_token.trim().is_empty(),
        "guild_id": st.cfg.bot_guild_id,
        "prefix": st.cfg.bot_prefix,
        "captcha": true,
        "last_setup": setup,
        "commands": ["help","ping","verify","check","status","snipe","ban","kick","mute","unmute","purge","slowmode","lock","unlock","userinfo","nuke"],
        "note": "gateway 24h roda como sidecar (BOT_TOKEN + BOT_GUILD_ID); setup cria a estrutura via web",
    }))
    .into_response()
}

#[derive(Debug, serde::Deserialize)]
struct BotSetup {
    guild_id: Option<u64>,
    owner_role: Option<String>,
    member_role: Option<String>,
    og_role: Option<String>,
}

async fn bot_setup(
    State(st): State<Arc<AppState>>,
    Json(body): Json<BotSetup>,
) -> impl IntoResponse {
    let token = st.cfg.bot_token.clone();
    if token.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "BOT_TOKEN ausente (env)"})),
        )
            .into_response();
    }
    let guild_id = body.guild_id.unwrap_or(st.cfg.bot_guild_id);
    if guild_id == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "guild_id ausente"})),
        )
            .into_response();
    }
    // Grupos (grupo → chaves de plataforma) a partir dos checkers nativos.
    let mut groups_map: HashMap<String, Vec<String>> = HashMap::new();
    for p in l96a1_checkers::all() {
        groups_map
            .entry(p.group().to_string())
            .or_default()
            .push(p.key().to_string());
    }
    let mut groups: Vec<(String, Vec<String>)> = groups_map.into_iter().collect();
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    let roles = l96a1_bot::RoleScheme {
        owner: body.owner_role.unwrap_or_else(|| "owner".into()),
        member: body.member_role.unwrap_or_else(|| "amigos".into()),
        og: body.og_role.unwrap_or_else(|| "og".into()),
    };
    match l96a1_bot::setup_guild(&token, guild_id, &groups, &roles).await {
        Ok(res) => {
            let summary = BotSetupSummary {
                ran_at: now_iso(),
                verify_channel_id: res.verify_channel_id,
                channels: res.channels.len(),
                log: res.log,
            };
            *st.bot_setup.lock().await = Some(summary.clone());
            Json(serde_json::json!({
                "ok": true,
                "summary": summary,
                "hits_url_configured": !res.hits_url.is_empty(),
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"ok": false, "error": e})),
        )
            .into_response(),
    }
}

// -------------------------------------------------------- fingerprint go

/// GET /go/:platform/:handle — 302 para a página do perfil (estilo fingerprint.to).
/// Só redireciona se o nick passar no validator; a UI só mostra o link em hits.
async fn go(Path((platform, handle)): Path<(String, String)>) -> impl IntoResponse {
    let plats = l96a1_checkers::all();
    let found = plats
        .iter()
        .find(|p| p.key().eq_ignore_ascii_case(&platform) || p.name().eq_ignore_ascii_case(&platform));
    match found {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "plataforma desconhecida"})),
        )
            .into_response(),
        Some(p) => match p.validator().normalize(&handle) {
            None => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "handle inválido para a plataforma"})),
            )
                .into_response(),
            Some(norm) => Redirect::temporary(&p.profile_url(&norm)).into_response(),
        },
    }
}

/// GET /v1/:platform/:handle — check único JSON (endpoint fingerprint-style).
async fn platform_handle(
    State(st): State<Arc<AppState>>,
    Path((platform, handle)): Path<(String, String)>,
) -> impl IntoResponse {
    let plats = l96a1_checkers::all();
    let found = plats
        .iter()
        .find(|p| p.key().eq_ignore_ascii_case(&platform) || p.name().eq_ignore_ascii_case(&platform));
    let Some(p) = found else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "plataforma desconhecida"})),
        )
            .into_response();
    };
    let Some(norm) = p.validator().normalize(&handle) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "handle inválido"})),
        )
            .into_response();
    };
    let router = l96a1_proxy::HybridRouter::default();
    let proxy = router.use_proxy(p.key(), p.profile().sticky_proxy, p.profile().needs_proxy, &st.rotator);
    let client = Arc::new(l96a1_core::build_client(
        Duration::from_secs(st.cfg.timeout_secs),
        None,
    ));
    let ctx = l96a1_core::CheckCtx {
        client,
        proxy_url: proxy.clone(),
        discord_tokens: st.discord_tokens.clone(),
        tiktok_sessions: st.tiktok_sessions.clone(),
        github_token: if st.cfg.github_token.is_empty() {
            None
        } else {
            Some(st.cfg.github_token.clone())
        },
        timeout: Duration::from_secs(st.cfg.timeout_secs),
    };
    let t = std::time::Instant::now();
    let o = p.check(&norm, &ctx).await;
    let profile_url = p.profile_url(&norm);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "platform": p.name(),
            "platformUrl": origin_of(&profile_url),
            "username": norm,
            "profileUrl": profile_url,
            "result": format!("{:?}", o.result).to_lowercase(),
            "tag": o.result.tag(),
            "latency_ms": o.latency_ms,
            "evidence": o.evidence,
            "elapsed_ms": t.elapsed().as_millis() as u64,
        })),
    )
        .into_response()
}

#[tokio::main]
async fn main() {
    // Carrega o `.env` da raiz (tokens, throttle, sniper) — sem isso,
    // `cargo run -p l96a1-api` já sobe com tudo do l96a1.toml migrado.
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let st = AppState::new();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Sniper 24h em background (equivale a `l9 snipe` em loop).
    {
        let st2 = st.clone();
        tokio::spawn(async move {
            loop {
                let wait = st2.cfg.snipe_interval_secs.max(60);
                tokio::time::sleep(Duration::from_secs(wait)).await;
                if st2.watchlist.lock().await.is_empty() {
                    continue;
                }
                run_snipe_cycle_and_notify(&st2).await;
            }
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/platforms", get(platforms))
        .route("/v1/catalogs", get(catalogs))
        .route("/v1/search", get(search_sse))
        .route("/v1/bulk", get(bulk_sse))
        .route("/v1/scans", post(create_scan).get(list_scans))
        .route("/v1/scans/{id}", get(get_scan))
        .route("/v1/scans/{id}/cancel", post(cancel_scan))
        .route("/v1/checked", get(checked_stats).delete(checked_clear))
        .route("/v1/checked/ttl", post(checked_ttl))
        .route("/v1/dataset/files", get(dataset_files))
        .route("/v1/dataset/scan", post(dataset_scan))
        .route("/v1/proxies", get(proxies_list))
        .route("/v1/proxies/test", post(proxies_test))
        .route("/v1/proxies/prune", post(proxies_prune))
        .route("/v1/proxies/unmask", get(proxies_unmask))
        .route("/v1/enrich", get(enrich_one_route))
        .route("/v1/enrich/batch", post(enrich_batch_route))
        .route("/v1/history", get(history).delete(history_delete))
        .route("/v1/rate-limit", get(rate_limit))
        .route("/v1/doctor", get(doctor))
        .route("/v1/snipe/status", get(snipe_status))
        .route("/v1/snipe/watchlist", get(snipe_watchlist).post(snipe_watchlist_edit))
        .route("/v1/snipe/run-once", post(snipe_run_once))
        .route("/v1/webhooks", get(webhooks))
        .route("/v1/webhooks/test", post(webhooks_test))
        .route("/v1/bot/status", get(bot_status))
        .route("/v1/bot/setup", post(bot_setup))
        .route("/go/{platform}/{handle}", get(go))
        .route("/v1/{platform}/{handle}", get(platform_handle))
        .layer(cors)
        .with_state(st);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("l96a1-api on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // Header SSE precisa de flush imediato; sem buffering no proxy.
    axum::serve(listener, app).await.unwrap();
}
