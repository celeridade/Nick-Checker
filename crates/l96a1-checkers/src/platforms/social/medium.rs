//! Medium: 200 renderiza view 404 p/ desconhecidos — só marcadores decidem.
//! Research (`research/medium.md`): `__typename:User` = taken;
//! `PAGE NOT FOUND`/`Out of nothing` = avail. Via nossa rede dá 403
//! (corpo sem marcadores) => RateLimited, nunca avail (403 verificado
//! sem miss-markers). Rota residencial p/ decisão real.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Medium;

/// Decisão pura da page.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 404 {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("web", "404")],
        )
    } else if body.contains("\"__typename\":\"User\"") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "user"),
                l96a1_core::Observation::taken_strong("web", "user2"),
            ],
        )
    } else if body.contains(">PAGE NOT FOUND<") || body.contains("Out of nothing, something.") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "404-view",
            )],
        )
    } else {
        common::finish(t, vec![l96a1_core::Observation::barrier("web", "ambiguous")])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Medium {
    fn key(&self) -> &'static str {
        "medium"
    }
    fn name(&self) -> &'static str {
        "Medium"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://medium.com/@{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://medium.com/@{user}");
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
    fn user_typename_is_taken() {
        let o = decide(200, r#"{"__typename":"User","name":"Medium"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn page_not_found_view_is_available() {
        let o = decide(200, "<div>PAGE NOT FOUND</div><div>Out of nothing, something.</div>", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn walled_403_is_never_available() {
        // REGRESSÃO: 403 sem marcadores não pode virar Available.
        let o = decide(403, "<html>blocked</html>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
