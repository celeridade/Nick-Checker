//! Probe compartilhado dos sites gerados (ex-motores `manifest`/`wmm`).
//!
//! Lógica idêntica aos antigos `ManifestChecker`/`WmnChecker`, só que os
//! dados vêm bakeds como consts em cada struct gerada (`platforms/<cat>/`)
//! em vez de JSON em runtime — cada site vira research-ready sem mudar
//! comportamento no dia 1.
//!
//! Hardening (audit 2026-10, research ao vivo):
//! - `response_url` HONRA `errorUrl` (antes ignorava e decidia só por
//!   status — kofi/codechef davam falso Taken em todo nick livre, pois
//!   missing redireciona p/ 200);
//! - códigos de barreira (401/403/5xx) NÃO decidem, exceto quando o
//!   próprio catálogo os declara como sinal (`errorCode`/`e_code`/`m_code`)
//!   — ex.: blogs WP.com usam 403 como taken desenhado;
//! - 429 vira `RateLimited` (antes caía em Taken/Available falsos);
//! - WMN: `dual-miss` NÃO é mais Available autoritativo — qualquer desvio
//!   (WAF, markup drift, redirect inesperado) era falso HIT. Agora só
//!   decidem: dual-match (taken), missing-string positiva (avail),
//!   missing-code desenhado (avail) ou redirect desenhado sem marcador
//!   (avail). Resto = Unknown honesto;
//! - marcadores WAF estreitados: `captcha` genérico dava falso RateLimited
//!   em pages legítimas com widget recaptcha (Medium/Spotify);
//! - corpo vazio em 2xx não decide (sem sinal).

use l96a1_core::{client_for, CheckCtx, CheckOutcome, Observation, UsernameValidator};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteErrorType {
    Message,
    StatusCode,
    ResponseUrl,
}

/// Dados bakeds de um site estilo Sherlock (gerado de `sites.json`).
pub struct BakedManifest {
    pub error_type: SiteErrorType,
    pub error_msg: &'static [&'static str],
    pub error_code: &'static [i64],
    pub regex: Option<&'static str>,
    pub method: &'static str,
    pub payload: Option<&'static str>,
    pub headers: &'static [(&'static str, &'static str)],
    pub url_probe: Option<&'static str>,
    /// Destino do redirect p/ missing (`response_url`; com `{}` p/ interpolar).
    pub error_url: Option<&'static str>,
}

/// Dados bakeds de um site estilo Blackbird/WMN (gerado de `wmn.json`).
pub struct BakedWmn {
    pub e_code: i64,
    pub e_string: &'static str,
    pub m_code: i64,
    pub m_string: &'static str,
}

pub fn interpolate(template: &str, user: &str) -> String {
    template.replace("{}", &user.replace(' ', "%20"))
}

/// Compara URLs finais ignorando `/` terminal (kofi responde `…/`).
pub fn same_url(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

fn interpolate_json(v: &serde_json::Value, user: &str) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => serde_json::Value::String(interpolate(s, user)),
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.iter().map(|x| interpolate_json(x, user)).collect())
        }
        serde_json::Value::Object(m) => serde_json::Value::Object(
            m.iter().map(|(k, x)| (k.clone(), interpolate_json(x, user))).collect(),
        ),
        _ => v.clone(),
    }
}

static REGEX_CACHE: OnceLock<Mutex<HashMap<&'static str, regex::Regex>>> = OnceLock::new();

fn cached_regex(pattern: &'static str) -> Option<regex::Regex> {
    let map = REGEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = map.lock() {
        if let Some(re) = guard.get(pattern) {
            return Some(re.clone());
        }
    }
    let re = regex::Regex::new(pattern).ok()?;
    if let Ok(mut guard) = map.lock() {
        guard.insert(pattern, re.clone());
    }
    Some(re)
}

/// Validator igual ao antigo motor: regex válida do catálogo ou `^.*$`.
pub fn validator_for(regex: Option<&'static str>) -> UsernameValidator {
    match regex {
        Some(p) if regex::Regex::new(p).is_ok() => UsernameValidator::new(1, 100, p),
        _ => UsernameValidator::new(1, 100, r"^.*$"),
    }
}

/// Marcadores de parede (minúsculos; comparar contra body lowercased).
/// Estreitos de propósito: `captcha` genérico foi REMOVIDO (widget
/// recaptcha existe em pages legítimas — Medium/Spotify dão falso
/// RateLimited). Só formas de challenge/parede entram aqui.
pub const WAF_MARKERS: &[&str] = &[
    "just a moment",
    "checking your browser",
    "verify you are human",
    "challenge-error-text",
    "cf-challenge",
    "cf_challenge",
    "attention required",
    "perimeterx",
    "awswafintegration",
    "request blocked",
    "access denied",
    "security check",
    "unusual activity",
];

/// Probe estilo Sherlock sobre dados bakeds (paridade com o motor antigo).
pub async fn probe_manifest(
    baked: &BakedManifest,
    url_template: &str,
    user: &str,
    ctx: &CheckCtx,
) -> CheckOutcome {
    let t = Instant::now();
    // pre-filtro regex do manifest (formato ilegal => Unknown, sem rede)
    if let Some(pattern) = baked.regex {
        let ok = cached_regex(pattern).map(|re| re.is_match(user)).unwrap_or(true);
        if !ok {
            return CheckOutcome::new(l96a1_core::CheckResult::Unknown, 0, "illegal-format");
        }
    }
    let target = baked
        .url_probe
        .map(|u| interpolate(u, user))
        .unwrap_or_else(|| interpolate(url_template, user));
    let client = client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let mut req = match baked.method {
        "POST" => client.post(&target),
        "HEAD" => client.head(&target),
        "PUT" => client.put(&target),
        _ => client.get(&target),
    };
    for (k, v) in baked.headers {
        req = req.header(*k, interpolate(v, user));
    }
    if let Some(p) = baked.payload {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(p) {
            req = req.json(&interpolate_json(&v, user));
        }
    }
    let (status, final_url, body) = match req.send().await {
        Ok(r) => {
            let s = r.status().as_u16();
            let u = r.url().as_str().to_string();
            (s, u, r.text().await.unwrap_or_default())
        }
        Err(_) => {
            return CheckOutcome::new(
                l96a1_core::CheckResult::Error,
                t.elapsed().as_millis() as u64,
                "network",
            )
        }
    };
    let ms = t.elapsed().as_millis() as u64;
    // 429 sempre barreira (nunca decide Taken/Available).
    if status == 429 {
        return CheckOutcome::new(l96a1_core::CheckResult::RateLimited, ms, "rate-429");
    }
    let lower = body.to_lowercase();
    if WAF_MARKERS.iter().any(|m| lower.contains(m)) {
        return CheckOutcome::new(l96a1_core::CheckResult::RateLimited, ms, "waf");
    }
    // Códigos de barreira/erro não decidem — EXCETO quando o catálogo os
    // declara como sinal (errorCode desenhado, ex.: Slides 204).
    let code_hit = baked.error_code.contains(&(status as i64));
    if matches!(status, 401 | 403) || (500..600).contains(&status) {
        if !code_hit {
            return CheckOutcome::new(l96a1_core::CheckResult::Unknown, ms, "blocked");
        }
    }
    // Corpo vazio em 2xx não prova perfil (meia-resposta, edge vazio).
    if body.is_empty() && (200..300).contains(&status) {
        return CheckOutcome::new(l96a1_core::CheckResult::Unknown, ms, "empty-body");
    }
    let obs = match baked.error_type {
        SiteErrorType::Message => {
            if baked.error_msg.is_empty() {
                // Sem marcador configurado não há sinal (nunca chutar).
                return CheckOutcome::new(l96a1_core::CheckResult::Unknown, ms, "no-marker");
            }
            if baked.error_msg.iter().any(|m| body.contains(m)) {
                vec![Observation::avail_authoritative("manifest", "errorMsg")]
            } else if status == 404 && body.is_empty() {
                // 404 sem corpo: nem marcador nem perfil — sem sinal.
                return CheckOutcome::new(l96a1_core::CheckResult::Unknown, ms, "bare-404");
            } else {
                vec![Observation::taken_authoritative("manifest", "no-errorMsg")]
            }
        }
        SiteErrorType::StatusCode => {
            if code_hit || !(200..300).contains(&status) {
                vec![Observation::avail_authoritative("manifest", "errorCode")]
            } else {
                vec![Observation::taken_authoritative("manifest", "2xx")]
            }
        }
        SiteErrorType::ResponseUrl => {
            // Missing redireciona p/ errorUrl (kofi, codechef, packagist…).
            // `errorUrl == url` no catálogo = dado inútil: cai no status.
            let err_hit = baked
                .error_url
                .map(|u| interpolate(u, user))
                .filter(|eu| *eu != target)
                .map(|eu| same_url(&final_url, &eu))
                .unwrap_or(false);
            if err_hit {
                vec![Observation::avail_authoritative("manifest", "errorUrl")]
            } else if (200..300).contains(&status) {
                vec![Observation::taken_authoritative("manifest", "2xx")]
            } else {
                vec![Observation::avail_authoritative("manifest", "non-2xx")]
            }
        }
    };
    CheckOutcome::new(l96a1_core::aggregate(&obs), ms, "manifest")
}

/// Probe estilo Blackbird/WMN sobre dados bakeds.
///
/// Regra dual (taken) + missing positivo (avail); qualquer outro estado é
/// Unknown honesto — o antigo `dual-miss => avail` gerava falso HIT em
/// WAF, markup drift e redirect inesperado.
pub async fn probe_wmn(
    baked: &BakedWmn,
    uri_template: &str,
    user: &str,
    ctx: &CheckCtx,
) -> CheckOutcome {
    let t = Instant::now();
    let url = interpolate(&uri_template.replace("{account}", "{}"), user);
    let client = client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let (status, final_url, body) = match client.get(&url).send().await {
        Ok(r) => {
            let s = r.status().as_u16() as i64;
            let u = r.url().as_str().to_string();
            (s, u, r.text().await.unwrap_or_default())
        }
        Err(_) => {
            return CheckOutcome::new(
                l96a1_core::CheckResult::Error,
                t.elapsed().as_millis() as u64,
                "network",
            )
        }
    };
    let ms = t.elapsed().as_millis() as u64;
    if status == 429 {
        return CheckOutcome::new(l96a1_core::CheckResult::RateLimited, ms, "rate-429");
    }
    let lower = body.to_lowercase();
    if WAF_MARKERS.iter().any(|m| lower.contains(m)) {
        return CheckOutcome::new(l96a1_core::CheckResult::RateLimited, ms, "waf");
    }
    // Barreira/erro não decide — exceto código DESENHADO no catálogo
    // (ex.: WP.com usa 403 como taken, Albicla usa 500).
    let designed = status == baked.e_code || status == baked.m_code;
    if (status == 401 || status == 403 || (500..600).contains(&status)) && !designed {
        return CheckOutcome::new(l96a1_core::CheckResult::Unknown, ms, "blocked");
    }
    let e_cfg = !baked.e_string.is_empty();
    let m_cfg = !baked.m_string.is_empty();
    // Cliente segue redirects: e_code/m_code 3xx viram "redirecionou".
    let redirected = !same_url(&final_url, &url);
    let e_match = e_cfg
        && body.contains(baked.e_string)
        && (!m_cfg || !body.contains(baked.m_string))
        && (status == baked.e_code
            || ((300..400).contains(&baked.e_code) && redirected));
    let obs = if e_match {
        vec![Observation::taken_authoritative("wmn", "dual-match")]
    } else if m_cfg && body.contains(baked.m_string) {
        // Missing-string POSITIVA na page final (vale p/ landing de
        // redirect, ex.: Pinterest -> home com `home-main-title`).
        vec![Observation::avail_authoritative("wmn", "missing-signal")]
    } else if baked.m_code != baked.e_code
        && status == baked.m_code
        && !(300..400).contains(&baked.m_code)
    {
        // Missing-code desenhado (ex.: 404 catalogado).
        vec![Observation::avail_authoritative("wmn", "missing-code")]
    } else if (300..400).contains(&baked.m_code) && redirected && !m_cfg {
        // Redirect puro desenhado sem marcador (ex.: 247CTF -> home).
        vec![Observation::avail_authoritative("wmn", "redirected")]
    } else {
        return CheckOutcome::new(l96a1_core::CheckResult::Unknown, ms, "ambiguous");
    };
    CheckOutcome::new(l96a1_core::aggregate(&obs), ms, "wmn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_replaces_placeholder_and_spaces() {
        assert_eq!(interpolate("https://x.to/user/{}/", "a b"), "https://x.to/user/a%20b/");
    }

    #[test]
    fn validator_accepts_anything_without_regex() {
        let v = validator_for(None);
        assert!(v.normalize("qualquer-nick_99").is_some());
    }

    #[test]
    fn same_url_ignores_trailing_slash() {
        assert!(same_url("https://ko-fi.com/", "https://ko-fi.com"));
        assert!(same_url("https://ko-fi.com", "https://ko-fi.com/"));
        assert!(!same_url("https://ko-fi.com/nick", "https://ko-fi.com/"));
        assert!(!same_url(
            "https://www.grailed.com/404",
            "https://www.grailed.com/adam"
        ));
    }

    #[test]
    fn waf_markers_are_narrow() {
        // REGRESSÃO: `captcha` genérico dava falso RateLimited em pages
        // legítimas com widget recaptcha (Medium/Spotify têm o texto).
        assert!(!WAF_MARKERS.contains(&"captcha"));
        assert!(!WAF_MARKERS.contains(&"recaptcha"));
        // …mas challenges reais continuam pegando.
        for m in ["just a moment", "cf-challenge", "unusual activity", "access denied"] {
            assert!(WAF_MARKERS.contains(&m), "{m}");
        }
    }
}
