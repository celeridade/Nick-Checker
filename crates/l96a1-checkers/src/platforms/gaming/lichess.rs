//! Lichess: GET /api/user/{u} — 200+id=taken, 404=avail.
//! Research (`research/lichess.md`): confirmado no DIRETO (proxy dá
//! timeout — manter `force_direct`). Follow-up `Banned`: engine/closed
//! têm flags públicas — mapear com exemplar.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct Lichess;

/// Decisão pura da API.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (200, b) if b.contains("\"id\"") => common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative("api", "id")],
        ),
        (404, _) => common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "404")],
        ),
        (s, b) => common::finish_http(t, s, b, &["\"id\""], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Lichess {
    fn key(&self) -> &'static str {
        "lichess"
    }
    fn name(&self) -> &'static str {
        "Lichess"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://lichess.org/@/{user}")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(2, 20, r"^[a-z0-9_-]+$")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://lichess.org/api/user/{user}");
        match common::get(ctx, &url, &[("Accept", "application/json")]).await {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => decide(s, &b, ms()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn id_is_taken() {
        let o = decide(200, r#"{"id":"thibault","username":"thibault"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn missing_404_is_available() {
        let o = decide(404, "Not found", 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
