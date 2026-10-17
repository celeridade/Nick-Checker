//! Helpers compartilhados dos checkers.

use l96a1_core::{aggregate, client_for, decide_http, CheckCtx, CheckOutcome, CheckResult};
use std::time::Instant;

/// GET simples; usa client com proxy do ctx quando definido.
pub async fn get(ctx: &CheckCtx, url: &str, extra_headers: &[(&str, &str)]) -> Option<(u16, String)> {
    let client = client_for(ctx.timeout, ctx.proxy_url.as_deref());
    let mut req = client.get(url);
    for (k, v) in extra_headers {
        req = req.header(*k, *v);
    }
    match req.send().await {
        Ok(r) => {
            let status = r.status().as_u16();
            let body = r.text().await.unwrap_or_default();
            Some((status, body))
        }
        Err(_) => None,
    }
}

pub fn outcome_net_err(lat: u64) -> CheckOutcome {
    CheckOutcome::new(CheckResult::Error, lat, "network/timeout")
}

/// GET sem seguir redirects; retorna (status, body, set-cookies).
/// P/ sites com self-redirect + cookie-jar manual (ex: guns.lol).
pub async fn get_nofollow(
    ctx: &CheckCtx,
    url: &str,
    extra_headers: &[(&str, &str)],
) -> Option<(u16, String, Vec<String>)> {
    let client = std::sync::Arc::new(l96a1_core::build_client_nofollow(
        ctx.timeout,
        ctx.proxy_url.as_deref(),
    ));
    let mut req = client.get(url);
    for (k, v) in extra_headers {
        req = req.header(*k, *v);
    }
    match req.send().await {
        Ok(r) => {
            let status = r.status().as_u16();
            let cookies: Vec<String> = r
                .headers()
                .get_all(reqwest::header::SET_COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .map(|s| s.split(';').next().unwrap_or("").to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let body = r.text().await.unwrap_or_default();
            Some((status, body, cookies))
        }
        Err(_) => None,
    }
}

pub fn finish(lat: Instant, obs: Vec<l96a1_core::Observation>) -> CheckOutcome {
    let ms = lat.elapsed().as_millis() as u64;
    let result = aggregate(&obs);
    let evidence = obs.iter().map(|o| format!("{}:{}", o.method, o.note)).collect::<Vec<_>>().join("|");
    CheckOutcome::new(result, ms, if evidence.is_empty() { "none".into() } else { evidence })
}

pub fn finish_http(lat: Instant, status: u16, body: &str, taken: &[&str], avail: &[&str]) -> CheckOutcome {
    // Marcadores WAF compartilhados com os gerados (audit 2026-10:
    // `captcha` genérico removido — pages legítimas com recaptcha).
    let obs = decide_http(status, body, taken, avail, crate::platforms::site_probe::WAF_MARKERS);
    finish(lat, obs)
}
