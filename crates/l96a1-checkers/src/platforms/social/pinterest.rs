//! Pinterest: JSON-LD Person = taken; `User not found.` (code 30) = avail.
//! Research (`research/pinterest.md`): livre caía em `ambiguous-200` =>
//! RateLimited (falso "rate" escondendo nick livre). Title vazio nos dois.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Pinterest;

/// Decisão pura da page (herdada por `mf:pinterest`/`wm:pinterest`).
pub(crate) fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 404
        && (body.contains("User not found.") || body.contains("\"code\":30"))
    {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "404-not-found",
            )],
        )
    } else if body.contains("\"@type\": \"Person\"") || body.contains("\"@type\":\"Person\"") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "person-ld"),
                l96a1_core::Observation::taken_strong("web", "person-ld2"),
            ],
        )
    } else if body.contains("User not found.") || body.contains("\"code\":30") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "user-not-found",
            )],
        )
    } else {
        // Shell/consent sem marcadores: Unknown honesto (nunca RateLimited).
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Pinterest {
    fn key(&self) -> &'static str {
        "pinterest"
    }
    fn name(&self) -> &'static str {
        "Pinterest"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.pinterest.com/{user}/")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 8, needs_proxy: true, sticky_proxy: true }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://www.pinterest.com/{user}/");
        match common::get(ctx, &url, &[]).await {
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
    fn person_ld_is_taken() {
        let o = decide(200, r#"<script type="application/ld+json">{"@type": "Person","name":"x"}</script>"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn user_not_found_is_available() {
        // REGRESSÃO: livre virava RateLimited (falso "rate").
        let o = decide(200, r#"{"code":30,"message":"User not found."}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn ambiguous_is_never_decided() {
        let o = decide(200, "<html>shell</html>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
