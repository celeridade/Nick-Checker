//! TikTok autenticado: pool de sessões + `GET /api/uniqueid/check/`.
//!
//! Espelho do modelo Discord (`gaming/discord.rs`): contas em
//! `tiktok_sessions.txt` (uma cookie-string por linha), rotação com
//! round-robin, cooldown por sessão em 429/wall e morte em 401, saúde
//! exposta p/ `/v1/doctor`. Sem sessões = custo zero (nem entra no caminho).
//!
//! Research (`research/tiktok.md`, CDP Brave-NOIA-Debug, conta @toxidez):
//! - Cliente chama `checkUniqueId(nick.lower)` com debounce 1s:
//!   `GET /api/uniqueid/check/?unique_id={nick}&...` + header `tt-csrf-token`.
//! - Semântica live: `{is_valid:true, status_code:0}` = LIVRE (check verde);
//!   `{is_valid:false, status_code:3249}` = ocupado/privado (não claimable).
//! - Assinatura ATUAL (variante da lib `@rediska1114/tiktok-api`, provada
//!   live): `&X-Bogus=` (double-MD5 + RC4 + base64 custom, magic
//!   `1245783967`) + `&X-Gnarly=` (ChaCha20 5.1.1) sobre a query exata,
//!   UA Safari/Mac. Sem login E sem assinatura o servidor devolve `{}` ou
//!   `3002301` — e sem sessão (mesmo assinado) devolve vazio: o pool de
//!   contas é ESSENCIAL, não opcional.
//!
//! CUSTO/RUNTIME: 1 GET leve (~1KB) por sessão tentada, no máximo 3
//! sessões por nick; sem `tiktok_sessions` o módulo nem é chamado.

use l96a1_core::{CheckCtx, CheckOutcome, CheckResult};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;

// ---------------------------------------------------------------- X-Bogus

/// Alfabeto custom do X-Bogus (base64 deslocado, sem o `=` de padding).
const XB_ALPHABET: &[u8; 64] =
    b"Dkdpgh4ZKsQB80/Mfvw36XI1R25-WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe";

/// RC4 clássico (KSA + PRGA), byte a byte.
fn xb_rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut d: [u8; 256] = [0; 256];
    for (i, b) in d.iter_mut().enumerate() {
        *b = i as u8;
    }
    let mut c: usize = 0;
    for i in 0..256 {
        c = (c + d[i] as usize + key[i % key.len()] as usize) % 256;
        d.swap(i, c);
    }
    let mut t: usize = 0;
    c = 0;
    let mut out = Vec::with_capacity(data.len());
    for &b in data {
        t = (t + 1) % 256;
        c = (c + d[t] as usize) % 256;
        d.swap(t, c);
        out.push(b ^ d[(d[t] as usize + d[c] as usize) % 256]);
    }
    out
}

fn xb_md5(data: &[u8]) -> [u8; 16] {
    md5::compute(data).0
}

/// Assina `(params, body)` no formato ATUAL do webmssdk (variante da lib
/// `@rediska1114/tiktok-api`: `encrypt(params, body, ua, ts_segundos)`).
/// Estrutura: `[64] + [0,1,14] + md5²(params)[14:16] + md5²(body)[14:16]`
/// `+ md5(b64(RC4(UA)))[14:16] + ts_BE + MAGIC_BE(1245783967)`, xor total
/// anexado, tudo sob RC4(`0xFF`) com prefixo `[2,255]`, base64 custom.
/// (Uma variante ANTIGA com magic `88,194,176,26` + interleave par/ímpar
/// circula em gists — o servidor a rejeita com `{}`/3002301.)
pub fn x_bogus(params: &str, body: &str, user_agent: &str, ts: u32) -> String {
    let r = xb_md5(&xb_md5(params.as_bytes()));
    let m = xb_md5(&xb_md5(body.as_bytes()));
    let e = xb_md5(
        base64::engine::general_purpose::STANDARD
            .encode(xb_rc4(&[0, 1, 14], user_agent.as_bytes()))
            .as_bytes(),
    );
    let mut b = Vec::with_capacity(19);
    b.push(64u8);
    b.extend_from_slice(&[0, 1, 14]);
    b.extend_from_slice(&r[14..16]);
    b.extend_from_slice(&m[14..16]);
    b.extend_from_slice(&e[14..16]);
    b.extend_from_slice(&ts.to_be_bytes());
    b.extend_from_slice(&1245783967u32.to_be_bytes());
    let y: u8 = b.iter().fold(0, |a, &x| a ^ x);
    b.push(y);
    let mut a = vec![2u8, 255];
    a.extend(xb_rc4(&[255], &b));
    let mut out = String::with_capacity(28);
    for i in (0..21).step_by(3) {
        let n = (a[i] as u32) << 16 | (a[i + 1] as u32) << 8 | a[i + 2] as u32;
        for j in [18, 12, 6, 0] {
            out.push(XB_ALPHABET[((n >> j) & 63) as usize] as char);
        }
    }
    out
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn now_ts() -> u32 {
    now_secs() as u32
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

// ---------------------------------------------------------------- X-Gnarly

/// X-Gnarly 5.1.1 (mesma versão que o `signUrl` funcional usa): envelope
/// `{md5(query), md5(body), md5(ua), ts, …}` cifrado em ChaCha20 de
/// rodadas dinâmicas, chave embutida no token, base64 custom.
/// O servidor extrai a chave do próprio token p/ decriptar — os dados
/// aleatórios só precisam ser bem-formados (qualquer u32 vale).
const XG_ALPHABET: &[u8; 64] =
    b"u09tbS3UvgDEe6r-ZVMXzLpsAohTn7mdINQlW412GqBjfYiyk8JORCF5/xKHwacP";
const XG_BUILD: &str = "1.0.0.314";
const XG_CANVAS: u32 = 1508145731;
const XG_CIPHER_INIT: [u32; 4] = [1196819126, 600974999, 3863347763, 1451689750];

fn xg_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

fn xg_rotl(v: u32, s: u32) -> u32 {
    v.rotate_left(s)
}

fn xg_quarter(st: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    st[a] = xg_add(st[a], st[b]);
    st[d] = xg_rotl(st[d] ^ st[a], 16);
    st[c] = xg_add(st[c], st[d]);
    st[b] = xg_rotl(st[b] ^ st[c], 12);
    st[a] = xg_add(st[a], st[b]);
    st[d] = xg_rotl(st[d] ^ st[a], 8);
    st[c] = xg_add(st[c], st[d]);
    st[b] = xg_rotl(st[b] ^ st[c], 7);
}

/// ChaCha de rodadas dinâmicas (colunas + diagonais alternadas, soma do
/// estado inicial ao final — variante do webmssdk).
fn xg_block(init: &[u32; 16], rounds: u32) -> [u32; 16] {
    let mut w = *init;
    let mut done = 0u32;
    while done < rounds {
        xg_quarter(&mut w, 0, 4, 8, 12);
        xg_quarter(&mut w, 1, 5, 9, 13);
        xg_quarter(&mut w, 2, 6, 10, 14);
        xg_quarter(&mut w, 3, 7, 11, 15);
        done += 1;
        if done >= rounds {
            break;
        }
        xg_quarter(&mut w, 0, 5, 10, 15);
        xg_quarter(&mut w, 1, 6, 11, 12);
        xg_quarter(&mut w, 2, 7, 12, 13);
        xg_quarter(&mut w, 3, 4, 13, 14);
        done += 1;
    }
    for i in 0..16 {
        w[i] = xg_add(w[i], init[i]);
    }
    w
}

/// Cifra simétrica por palavras LE (decifrar = cifrar de novo com o mesmo
/// estado inicial — usado no teste de round-trip).
fn xg_crypt_words(key: &[u32; 16], rounds: u32, words: &mut [u32]) {
    let mut state = *key;
    let mut off = 0usize;
    while off + 16 < words.len() {
        let ks = xg_block(&state, rounds);
        state[12] = xg_add(state[12], 1);
        for k in 0..16 {
            words[off + k] ^= ks[k];
        }
        off += 16;
    }
    let ks = xg_block(&state, rounds);
    for k in 0..words.len() - off {
        words[off + k] ^= ks[k];
    }
}

fn xg_crypt_bytes(key: &[u32; 16], rounds: u32, data: &mut Vec<u8>) {
    let n_words = data.len().div_ceil(4);
    let mut words = vec![0u32; n_words];
    for (i, w) in words.iter_mut().enumerate() {
        let mut v = 0u32;
        for k in 0..4 {
            if let Some(&b) = data.get(i * 4 + k) {
                v |= (b as u32) << (8 * k);
            }
        }
        *w = v;
    }
    xg_crypt_words(key, rounds, &mut words);
    data.clear();
    for w in words {
        data.extend_from_slice(&w.to_le_bytes());
    }
}

/// Inteiro -> 2 ou 4 bytes BE (limiar 65025).
fn xg_int_bytes(v: u32) -> Vec<u8> {
    if v < 65025 {
        vec![(v >> 8) as u8, v as u8]
    } else {
        v.to_be_bytes().to_vec()
    }
}

/// Primeiros 4 bytes UTF-8 como u32 BE.
fn xg_str_head(s: &str) -> u32 {
    let mut acc = 0u32;
    for b in s.as_bytes().iter().take(4) {
        acc = (acc << 8) | *b as u32;
    }
    acc
}

fn xg_md5hex(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let d = xb_md5(data);
    let mut s = String::with_capacity(32);
    for b in d {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 15) as usize] as char);
    }
    s
}

/// Fonte de u32 do token (produção: tempo+contador; testes: sequência fixa).
fn xg_rand_u32() -> u32 {
    static STATE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = STATE.fetch_add(0x9E3779B97F4A7C15, std::sync::atomic::Ordering::Relaxed);
    let t = now_ms();
    // xorshift64* sobre (tempo ^ contador).
    let mut x = t ^ n;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    (x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u32
}

/// Gera o X-Gnarly 5.1.1 p/ `(query, body, ua, ts_ms)`.
pub fn x_gnarly(query: &str, body: &str, user_agent: &str, ts_ms: u64) -> String {
    x_gnarly_with_rand(query, body, user_agent, ts_ms, &xg_rand_u32)
}

fn x_gnarly_with_rand(
    query: &str,
    body: &str,
    user_agent: &str,
    ts_ms: u64,
    rnd: &dyn Fn() -> u32,
) -> String {
    // Envelope (ordens exatas do JS): chaves 1..11, checksum 12, chave 0.
    // Enum preserva a distinção número/string dos dois checksums.
    enum V {
        Num(u32),
        Str(Vec<u8>),
    }
    let mut keys: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let mut vals: Vec<V> = vec![
        V::Num(1),
        V::Num(0),
        V::Str(xg_md5hex(query.as_bytes()).into_bytes()),
        V::Str(xg_md5hex(body.as_bytes()).into_bytes()),
        V::Str(xg_md5hex(user_agent.as_bytes()).into_bytes()),
        V::Num((ts_ms / 1000) as u32),
        V::Num(XG_CANVAS),
        V::Num((ts_ms as u128 * 1000 % 2147483648) as u32),
        V::Str(b"5.1.1".to_vec()),
        V::Str(XG_BUILD.as_bytes().to_vec()),
        V::Num(1),
    ];
    // checksum12: números crus, strings via head de 4 bytes.
    let mut checksum = 0u32;
    for v in vals.iter() {
        checksum ^= match v {
            V::Num(n) => *n,
            V::Str(s) => xg_str_head(std::str::from_utf8(s).unwrap_or("")),
        };
    }
    keys.push(12);
    vals.push(V::Num(checksum));
    // Checksum final: SÓ numéricos das chaves 1..12 (inclui o próprio 12).
    let mut final_sum = 0u32;
    for v in vals.iter() {
        if let V::Num(n) = v {
            final_sum ^= *n;
        }
    }
    keys.push(0);
    vals.push(V::Num(final_sum));
    // Serialização: [n_chaves] + [chave, len-BE, bytes]… (ordem de inserção).
    let mut payload = vec![keys.len() as u8];
    for (k, v) in keys.iter().zip(vals.iter()) {
        let bytes = match v {
            V::Num(n) => xg_int_bytes(*n),
            V::Str(s) => s.clone(),
        };
        payload.push(*k as u8);
        payload.extend_from_slice(&xg_int_bytes(bytes.len() as u32));
        payload.extend_from_slice(&bytes);
    }
    // Chave de cifra (12 palavras) + rodadas dinâmicas.
    let mut key_words = [0u32; 12];
    let mut acc = 0u32;
    for w in key_words.iter_mut() {
        *w = rnd();
        acc = (acc + (*w & 15)) & 15;
    }
    let rounds = acc + 5;
    let mut state = [0u32; 16];
    state[..4].copy_from_slice(&XG_CIPHER_INIT);
    state[4..16].copy_from_slice(&key_words);
    let mut data = payload.clone();
    xg_crypt_bytes(&state, rounds, &mut data);
    // Posição de inserção da chave.
    let mut key_bytes = Vec::with_capacity(48);
    for w in key_words {
        key_bytes.extend_from_slice(&w.to_le_bytes());
    }
    let mut pos = 0usize;
    for &b in key_bytes.iter() {
        pos = (pos + b as usize) % (data.len() + 1);
    }
    for &b in data.iter() {
        pos = (pos + b as usize) % (data.len() + 1);
    }
    let mut final_b = vec![b'K'];
    final_b.extend_from_slice(&data[..pos]);
    final_b.extend_from_slice(&key_bytes);
    final_b.extend_from_slice(&data[pos..]);
    // Base64 custom, só blocos cheios (resto descartado, como no JS).
    let mut out = String::new();
    let full = final_b.len() / 3 * 3;
    for i in (0..full).step_by(3) {
        let n = (final_b[i] as u32) << 16 | (final_b[i + 1] as u32) << 8 | final_b[i + 2] as u32;
        for j in [18, 12, 6, 0] {
            out.push(XG_ALPHABET[((n >> j) & 63) as usize] as char);
        }
    }
    out
}

// ---------------------------------------------------------------- cookies

/// Valor de um cookie dentro da cookie-string da sessão.
fn cookie_get<'a>(session: &'a str, name: &str) -> Option<&'a str> {
    session.split(';').map(str::trim).find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        if k.trim() == name {
            Some(v.trim())
        } else {
            None
        }
    })
}

/// CSRF que o app manda como header `tt-csrf-token`.
fn csrf_token(session: &str) -> Option<&str> {
    cookie_get(session, "tt_csrf_token").or_else(|| cookie_get(session, "csrftoken"))
}

/// `device_id`/`odinId` frescos por request (`7` + 18 dígitos, como a lib
/// oficial faz a cada chamada — estabilidade entre requests não é exigida).
fn fresh_device_id() -> String {
    let hi = xg_rand_u32() as u64;
    let lo = xg_rand_u32() as u64;
    format!("7{:018}", ((hi << 18) | lo) % 1_000_000_000_000_000_000u64)
}

// ---------------------------------------------------------------- pool

#[derive(Debug, Clone, Default)]
struct SessionState {
    limited_until: u64,
    /// 0 = viva; senão epoch até quando está aposentada (auto-cura).
    dead_until: u64,
}

const DEAD_TTL_SECS: u64 = 3600;
/// Teto de tentativas autenticadas por nick (1 request leve cada).
const MAX_SESSION_ATTEMPTS: usize = 3;

static SESSION_POOL: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, SessionState>>> =
    std::sync::OnceLock::new();

fn pool() -> &'static std::sync::Mutex<std::collections::HashMap<String, SessionState>> {
    SESSION_POOL.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Cursor atômico do round-robin (espalha o bulk entre as contas em vez
/// de empilhar tudo na primeira viva — padrão suenerve).
static RR_CURSOR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Id curto e estável p/ logs/saúde (nunca vaza cookie).
pub fn session_id(session: &str) -> String {
    let mut h = DefaultHasher::new();
    session.hash(&mut h);
    format!("sess-{:08x}", h.finish() as u32)
}

fn is_usable(map: &std::collections::HashMap<String, SessionState>, id: &str, now: u64) -> bool {
    match map.get(id) {
        None => true,
        Some(st) => st.dead_until <= now && st.limited_until <= now,
    }
}

/// Sessões utilizáveis agora, em ordem rotativa (round-robin).
fn usable_sessions(sessions: &[String]) -> Vec<String> {
    let cleaned: Vec<String> =
        sessions.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    if cleaned.is_empty() {
        return Vec::new();
    }
    let now = now_secs();
    let map = pool().lock().unwrap();
    let alive: Vec<String> =
        cleaned.into_iter().filter(|s| is_usable(&map, &session_id(s), now)).collect();
    if alive.is_empty() {
        return Vec::new();
    }
    let start =
        RR_CURSOR.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % alive.len();
    alive.iter().cycle().skip(start).take(alive.len()).cloned().collect()
}

/// Pausa a conta (429/wall). Paredes longas pausam só a conta.
fn note_session_ratelimited(session: &str, secs: f64) {
    if secs < 1.0 {
        return;
    }
    let until = now_secs() + (secs as u64).min(3600);
    pool().lock().unwrap().entry(session_id(session)).or_default().limited_until = until;
}

/// Aposenta a conta (só 401 de verdade — erro de rede/timeout de proxy
/// NUNCA mata conta).
fn note_session_dead(session: &str) {
    let until = now_secs() + DEAD_TTL_SECS;
    pool().lock().unwrap().entry(session_id(session)).or_default().dead_until = until;
}

/// Saúde por conta p/ `/v1/doctor` (só id mascarado, sem segredo).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionHealth {
    pub id: String,
    pub state: &'static str,
    pub retry_in_secs: u64,
}

pub fn session_health(sessions: &[String]) -> Vec<SessionHealth> {
    let now = now_secs();
    let map = pool().lock().unwrap();
    sessions
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let (state, retry_in_secs) = match map.get(&session_id(s)) {
                None => ("ok", 0),
                Some(st) if st.dead_until > now => ("dead", 0),
                Some(st) if st.limited_until > now => ("limited", st.limited_until - now),
                _ => ("ok", 0),
            };
            SessionHealth { id: session_id(s), state, retry_in_secs }
        })
        .collect()
}

// ---------------------------------------------------------------- request

/// UA fixo do checker (entra na assinatura e no param `browser_version`;
/// deve ser o mesmo nos dois — divergência queima a assinatura).
/// Safari/Mac = receita provada live (lib rediska + uniqueid/check).
pub const SESSION_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Safari/605.1.15";

/// Query na forma EXATA provada live (modal Edit profile + lib oficial):
/// `unique_id` snake, `user_is_login=true` (sessão), sem `msToken`/`secUid`.
/// Ordem = ordem de serialização (a assinatura cobre a string exata).
fn build_query(nick_lower: &str, _session: &str) -> String {
    let nick_lower = nick_lower.to_lowercase();
    let pairs = [
        ("WebIdLastTime", now_ms().to_string()),
        ("abTestVersion", String::new()),
        ("aid", "1988".to_string()),
        ("appType", "m".to_string()),
        ("app_language", "en-GB".to_string()),
        ("app_name", "tiktok_web".to_string()),
        ("browser_language", "en-GB".to_string()),
        ("browser_name", "Mozilla".to_string()),
        ("browser_online", "true".to_string()),
        ("browser_platform", "MacIntel".to_string()),
        ("browser_version", SESSION_UA.to_string()),
        ("channel", "tiktok_web".to_string()),
        ("cookie_enabled", "true".to_string()),
        ("data_collection_enabled", "true".to_string()),
        ("device_id", fresh_device_id()),
        ("device_platform", "web_pc".to_string()),
        ("focus_state", "true".to_string()),
        ("from_page", "user".to_string()),
        ("history_len", "6".to_string()),
        ("is_fullscreen", "false".to_string()),
        ("is_page_visible", "true".to_string()),
        ("language", "en-GB".to_string()),
        ("odinId", fresh_device_id()),
        ("os", "mac".to_string()),
        ("priority_region", String::new()),
        ("referer", String::new()),
        ("region", "GB".to_string()),
        ("screen_height", "915".to_string()),
        ("screen_width", "1052".to_string()),
        ("tz_name", "Europe/London".to_string()),
        ("unique_id", nick_lower),
        ("user_is_login", "true".to_string()),
        ("webcast_language", "en-GB".to_string()),
        ("needAudienceControl", "true".to_string()),
    ];
    pairs.iter().map(|(k, v)| format!("{}={}", k, form_encode(v))).collect::<Vec<_>>().join("&")
}

/// Percent-encode idêntico ao `URLSearchParams` do JS (é sobre ESSA string
/// que as assinaturas são calculadas): `A-Za-z0-9*-._` crus, espaço `+`,
/// resto `%XX` maiúsculo.
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'*') {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Decisão pura do `/api/uniqueid/check/`. Só veredito REAL decide:
/// `{is_valid:true, status_code:0}` = LIVRE (check verde do app);
/// `is_valid:false` = ocupado (formato já pré-validado pelo `validator()`
/// 2-24, então falso = taken). `3002301`/vazio/ambíguo = Erro/Unknown
/// (fallthrough p/ oembed+page — nunca decide errado).
fn decide_authed(status: u16, headers: &reqwest::header::HeaderMap, body: &str, ms: u64) -> CheckOutcome {
    let mut out = if status == 401 {
        CheckOutcome::new(CheckResult::Error, 0, "unauthorized")
    } else if status == 429 || body.contains("retry_after") {
        let secs = headers
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse::<f64>().ok())
            .or_else(|| {
                serde_json::from_str::<serde_json::Value>(body)
                    .ok()
                    .and_then(|v| v.get("retry_after")?.as_f64())
            })
            .unwrap_or(60.0);
        CheckOutcome::new(CheckResult::RateLimited, 0, format!("retry_after session {secs:.1}s"))
    } else if body.contains("3002301") || body.contains("try again later") {
        // Muro anti-bot transitório: pausa a conta, engine retenta noutro proxy.
        CheckOutcome::new(CheckResult::RateLimited, 0, "wall-3002301 60.0s")
    } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        let valid = v.get("is_valid").and_then(|x| x.as_bool());
        let code = v.get("status_code").and_then(|x| x.as_i64()).unwrap_or(-1);
        match (valid, code) {
            (Some(true), 0) => CheckOutcome::new(CheckResult::Available, 0, "uniqueid-check:free"),
            (Some(false), _) => CheckOutcome::new(CheckResult::Taken, 0, "uniqueid-check:taken"),
            _ => CheckOutcome::new(CheckResult::Unknown, 0, "verdict-unclear"),
        }
    } else {
        // `{}` / vazio / HTML: sem assinatura válida ou sessão fria.
        CheckOutcome::new(CheckResult::Unknown, 0, "no-verdict")
    };
    out.latency_ms = ms;
    out
}

fn retry_secs_evidence(evidence: &str) -> f64 {
    // Formatos: `retry_after session 12.3s` | `wall-3002301 60.0s`.
    evidence
        .split_whitespace()
        .next_back()
        .and_then(|t| t.trim_end_matches('s').parse::<f64>().ok())
        .unwrap_or(60.0)
}

async fn attempt_authed(user: &str, session: &str, ctx: &CheckCtx, ms: u64) -> CheckOutcome {
    let csrf = match csrf_token(session) {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => return CheckOutcome::new(CheckResult::Unknown, ms, "no-csrf"),
    };
    // Receita provada live (node-sign + sessão + csrf): query app-shape,
    // `&X-Bogus=` + `&X-Gnarly=` anexados, sem login = vazio.
    let query = build_query(&user.to_lowercase(), session);
    let ts_sec = now_ts();
    let signed = format!(
        "{query}&X-Bogus={}&X-Gnarly={}",
        x_bogus(&query, "", SESSION_UA, ts_sec),
        x_gnarly(&query, "", SESSION_UA, ts_sec as u64 * 1000)
    );
    let url = format!("https://www.tiktok.com/api/uniqueid/check/?{signed}");
    let client = l96a1_core::client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let resp = client
        .get(&url)
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "en-GB,en;q=0.9")
        .header("Referer", "https://www.tiktok.com/")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header(reqwest::header::USER_AGENT, SESSION_UA)
        .header("tt-csrf-token", csrf)
        .header("Cookie", session.trim())
        .send()
        .await;
    match resp {
        Err(_) => crate::common::outcome_net_err(ms),
        Ok(r) => {
            let st = r.status().as_u16();
            let hd = r.headers().clone();
            let tx = r.text().await.unwrap_or_default();
            decide_authed(st, &hd, &tx, ms)
        }
    }
}

/// Tenta o pool (round-robin, até `MAX_SESSION_ATTEMPTS` sessões).
/// Retorna `Some` só p/ veredito conclusivo (Taken/Available) ou rate
/// (p/ backoff honesto); `None` = cai no fallback unauthed (oembed+page).
pub async fn attempt_pool(user: &str, ctx: &CheckCtx, ms: u64) -> Option<CheckOutcome> {
    let usable = usable_sessions(&ctx.tiktok_sessions);
    if usable.is_empty() {
        return None;
    }
    let mut last_rate: Option<CheckOutcome> = None;
    for session in usable.iter().take(MAX_SESSION_ATTEMPTS.min(usable.len())) {
        let o = attempt_authed(user, session, ctx, ms).await;
        match o.result {
            CheckResult::Taken | CheckResult::Available => return Some(o),
            CheckResult::RateLimited => {
                note_session_ratelimited(session, retry_secs_evidence(&o.evidence));
                last_rate = Some(o);
            }
            CheckResult::Error if o.evidence == "unauthorized" => {
                note_session_dead(session);
            }
            _ => {}
        }
    }
    last_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UA exato do vetor de referência (lib rediska): Safari/Mac — a
    /// assinatura amarra o UA byte a byte.
    const VEC_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Safari/605.1.15";

    const TEST_UA: &str = SESSION_UA;

    #[test]
    fn xbogus_matches_reference_vector() {
        // Vetor gerado pelo `encrypt()` DA LIB que funciona (node, ts fixo),
        // comparado por BYTES. Mesmos inputs => mesmos bytes.
        let xb = x_bogus("unique_id=tiktok&aid=1988", "", VEC_UA, 1791478608);
        assert_eq!(
            xb.bytes().collect::<Vec<_>>(),
            vec![
                68, 70, 83, 122, 115, 119, 86, 79, 45, 78, 88, 65, 78, 72, 104, 108,
                67, 49, 56, 77, 77, 71, 108, 85, 114, 110, 48, 76
            ]
        );
    }

    #[test]
    fn xbogus_shape_is_28_alphabet_chars() {
        let xb = x_bogus("unique_id=x&aid=1988", "", TEST_UA, 1791478608);
        assert_eq!(xb.len(), 28);
        assert!(xb.bytes().all(|b| XB_ALPHABET.contains(&b)));
    }

    #[test]
    fn xbogus_binds_query_and_time() {
        let a = x_bogus("unique_id=x&aid=1988", "", TEST_UA, 1791478608);
        let b = x_bogus("unique_id=y&aid=1988", "", TEST_UA, 1791478608);
        let c = x_bogus("unique_id=x&aid=1988", "", TEST_UA, 1791478609);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    fn hdrs() -> reqwest::header::HeaderMap {
        reqwest::header::HeaderMap::new()
    }

    #[test]
    fn authed_free_pair_decides() {
        // Check verde do app: is_valid + code 0 = Available autoritativo
        // (mata falso positivo do fallback).
        let o = decide_authed(200, &hdrs(), r#"{"status_code":0,"is_valid":true}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
        // Ocupado: formato já pré-validado => Taken autoritativo.
        let o = decide_authed(200, &hdrs(), r#"{"status_code":0,"is_valid":false}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn authed_never_decides_without_verdict() {
        // `{}` / vazio / 3002301 / ambíguo => nunca Taken/Available.
        for b in ["{}", "", r#"{"status_code":3002301,"status_msg":"Please try again later"}"#] {
            let o = decide_authed(200, &hdrs(), b, 5);
            assert!(
                matches!(o.result, CheckResult::Unknown | CheckResult::RateLimited),
                "{b}"
            );
        }
        let o = decide_authed(200, &hdrs(), r#"{"status_code":4,"is_valid":true}"#, 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn authed_rate_and_dead_map() {
        let o = decide_authed(429, &hdrs(), r#"{"retry_after":65}"#, 5);
        assert_eq!(o.result, CheckResult::RateLimited);
        let o = decide_authed(401, &hdrs(), "{}", 5);
        assert_eq!(o.result, CheckResult::Error);
    }

    #[test]
    fn pool_helpers_behave() {
        assert_eq!(cookie_get("a=1; tt_csrf_token=ABC; b=2", "tt_csrf_token"), Some("ABC"));
        assert_eq!(cookie_get("a=1; csrftoken=XYZ", "tt_csrf_token"), None);
        let d1 = fresh_device_id();
        assert_eq!(d1.len(), 19);
        assert!(d1.starts_with('7'));
        let q = build_query("Quilometragem", "odin_tt=1; region=BR");
        // Nick lowercased, forma app-shape; device/odin frescos por request.
        assert!(q.contains("unique_id=quilometragem"));
        assert!(q.contains("aid=1988") && q.contains("user_is_login=true"));
        assert!(q.contains("odinId=7"));
    }

    #[test]
    fn xgnarly_crypt_roundtrip() {
        // Simetria do stream: cifrar 2x com mesma chave/rodadas recupera.
        // Exercita quarter-rounds, blocos, contador e caudas parciais.
        let key = [
            1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        ];
        let mut words: Vec<u32> =
            (0..20u32).map(|i| i.wrapping_mul(2654435761).wrapping_add(7)).collect();
        let orig = words.clone();
        xg_crypt_words(&key, 9, &mut words);
        assert_ne!(words, orig);
        xg_crypt_words(&key, 9, &mut words);
        assert_eq!(words, orig);
    }

    #[test]
    fn xgnarly_token_shape_with_fixed_dice() {
        // Dados fixos => token determinístico e bem-formado (base64 custom,
        // tamanho múltiplo de 4). O valor exato só o servidor valida —
        // a prova real é o veredito live.
        let t1 = x_gnarly_with_rand("aid=1988&unique_id=x", "", TEST_UA, 1791491697000, &|| {
            0x12345678
        });
        let t2 = x_gnarly_with_rand("aid=1988&unique_id=x", "", TEST_UA, 1791491697000, &|| {
            0x12345678
        });
        assert_eq!(t1, t2);
        assert!(!t1.is_empty() && t1.len() % 4 == 0);
        assert!(t1.bytes().all(|b| XG_ALPHABET.contains(&b)));
        let t3 = x_gnarly_with_rand("aid=1988&unique_id=y", "", TEST_UA, 1791491697000, &|| {
            0x12345678
        });
        assert_ne!(t1, t3);
    }

    #[test]
    fn pool_skips_limited_and_dead() {
        let a = "sess-pool-A x".to_string();
        let b = "sess-pool-B y".to_string();
        assert_eq!(usable_sessions(&[a.clone(), b.clone()]).len(), 2);
        note_session_ratelimited(&a, 600.0);
        let u = usable_sessions(&[a.clone(), b.clone()]);
        assert_eq!(u, vec![b.clone()]);
        note_session_dead(&b);
        assert!(usable_sessions(&[a.clone(), b.clone()]).is_empty());
        let h = session_health(&[a.clone(), b.clone()]);
        assert_eq!(h[0].state, "limited");
        assert_eq!(h[1].state, "dead");
        assert!(h[0].id.starts_with("sess-"));
    }
}
