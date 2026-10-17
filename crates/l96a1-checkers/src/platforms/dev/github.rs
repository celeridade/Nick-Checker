//! GitHub: GET api.github.com/users/{u} — 404=avail, 200+id=taken.
//! Research (`research/github.md`): confirmado. Caveat: conta SUSPENSA
//! também dá 404 na API (falso Available potencial) — follow-up com page
//! como 2º sinal p/ nicks de valor. Validator GH real (1-39, sem hífen
//! nas pontas).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, Observation, UsernameValidator};
use std::time::Instant;

pub struct Github;

/// Decisão pura da API.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (404, _) => common::finish(
            t,
            vec![Observation::avail_authoritative("api", "404")],
        ),
        (200, b) if b.contains("\"id\"") => common::finish(
            t,
            vec![Observation::taken_authoritative("api", "id")],
        ),
        (403, _) | (429, _) => common::finish(
            t,
            vec![Observation::barrier("api", "rate-limited")],
        ),
        (s, b) => common::finish_http(t, s, b, &["\"id\""], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Github {
    fn key(&self) -> &'static str {
        "github"
    }
    fn name(&self) -> &'static str {
        "GitHub"
    }
    fn group(&self) -> &'static str {
        "dev"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://github.com/{user}")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(1, 39, r"^[a-z0-9](?:[a-z0-9-]{0,37}[a-z0-9])?$")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://api.github.com/users/{user}");
        let client = l96a1_core::client_for(ctx.timeout, ctx.proxy_url.as_deref());
        let mut req = client.get(&url).header("Accept", "application/vnd.github+json");
        if let Some(tok) = &ctx.github_token {
            if !tok.is_empty() {
                req = req.header("Authorization", format!("Bearer {tok}"));
            }
        }
        match req.send().await {
            Err(_) => common::outcome_net_err(ms()),
            Ok(r) => {
                let s = r.status().as_u16();
                decide(s, &r.text().await.unwrap_or_default(), ms())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn taken_has_id() {
        let o = decide(200, r#"{"login":"torvalds","id":1024025}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn missing_404_is_available() {
        let o = decide(404, r#"{"message":"Not Found"}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn rate_is_barrier() {
        let o = decide(403, "{}", 5);
        assert_eq!(o.result, CheckResult::RateLimited);
    }
}
