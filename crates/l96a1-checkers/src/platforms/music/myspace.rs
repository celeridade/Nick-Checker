//! Myspace: 404=avail; 200 + title self-ref `(nick) on Myspace` = taken.
//! Research (`research/myspace.md`): marcadores antigos
//! (`profileHeading`/`profile-user`) MORTOS — ocupado virava RateLimited.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Myspace;

/// Decisão pura da page (herdada por `mf:myspace`/`wm:myspace`).
pub(crate) fn decide(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    let mut out = if status == 404 {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("web", "404")],
        )
    } else if status == 200 && body.contains("Page Not Found") {
        // View 404 servida em 200 (verificado ao vivo 2026-10): livre.
        // (Mesma classe do falso-RateLimited do Pinterest.)
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "not-found-view",
            )],
        )
    } else if status == 200 && lower.contains(&format!("({u}) on myspace")) {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "title-selfref"),
                l96a1_core::Observation::taken_strong("web", "title-selfref2"),
            ],
        )
    } else {
        common::finish_http(t, status, body, &[" on myspace"], &[])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Myspace {
    fn key(&self) -> &'static str {
        "myspace"
    }
    fn name(&self) -> &'static str {
        "Myspace"
    }
    fn group(&self) -> &'static str {
        "music"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://myspace.com/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://myspace.com/{user}");
        match common::get(ctx, &url, &[]).await {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => decide(user, s, &b, ms()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn title_selfref_is_taken() {
        let o = decide("tom", 200, "<title>Tom Anderson (tom) on Myspace</title>", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn missing_404_is_available() {
        let o = decide("x", 404, "nope", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn not_found_view_on_200_is_available() {
        // REGRESSÃO (research 2026-10): view 404-em-200 caía em
        // ambiguous => RateLimited, escondendo nick livre.
        let o = decide("x", 200, "<h1>Page Not Found</h1>", 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
