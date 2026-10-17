//! Imgur: account API — 200+id=taken, 404/400=avail.
//! Research (`research/imgur.md`): API instável daqui (timeout/429
//! over-capacity em todas as rotas). Sem fixture válida: sem mudança de
//! classificação até validar. `client_id` pode estar queimado (alternar).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Imgur;

/// Decisão pura da API.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (200, b) if b.contains("\"id\"") => common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative("api", "id")],
        ),
        (404, _) | (400, _) => common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "miss")],
        ),
        (s, b) => common::finish_http(t, s, b, &["\"id\""], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Imgur {
    fn key(&self) -> &'static str {
        "imgur"
    }
    fn name(&self) -> &'static str {
        "Imgur"
    }
    fn group(&self) -> &'static str {
        "community"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://imgur.com/user/{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!(
            "https://api.imgur.com/account/v1/accounts/{user}?client_id=546c25a59c58ad7"
        );
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
        let o = decide(200, r#"{"data":{"id":123},"success":true}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn miss_is_available() {
        assert_eq!(decide(404, "{}", 5).result, CheckResult::Available);
        assert_eq!(decide(400, "{}", 5).result, CheckResult::Available);
    }

    #[test]
    fn over_capacity_is_never_decided() {
        let o = decide(403, r#"{"data":{"error":"over capacity"},"success":false}"#, 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
