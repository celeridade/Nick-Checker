//! SoundCloud: 404=avail; 200 + og:title + username-key + self-ref = taken.
//! Research (`research/soundcloud.md`): marcadores antigos
//! (`soundcloud://users:`, `"username":"<nick>"`) MORTOS — ocupado virava
//! RateLimited. 403 segue barreira.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Soundcloud;

/// Decisão pura da page (herdada por `mf:soundcloud`/`wm:soundcloud`).
pub(crate) fn decide(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    // Página de missing ("We can't find that user.") pode vir em 200 ou
    // 404 (verificado ao vivo 2026-10): texto decide, nunca o shell.
    // Substring sem apóstrofo (reta/curva tanto faz).
    let not_found = body.contains("find that user");
    let mut out = if (status == 404 || status == 200) && not_found {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "not-found",
            )],
        )
    } else if status == 404 {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("web", "404")],
        )
    } else if status == 200
        && body.contains("og:title")
        && body.contains("\"username\"")
        && lower.contains(&u)
    {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "og-username"),
                l96a1_core::Observation::taken_strong("web", "selfref"),
            ],
        )
    } else if status == 403 {
        common::finish(t, vec![l96a1_core::Observation::barrier("web", "403")])
    } else {
        common::finish_http(
            t,
            status,
            body,
            &["og:title"],
            &[],
        )
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Soundcloud {
    fn key(&self) -> &'static str {
        "soundcloud"
    }
    fn name(&self) -> &'static str {
        "SoundCloud"
    }
    fn group(&self) -> &'static str {
        "music"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://soundcloud.com/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://soundcloud.com/{user}");
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
    fn og_title_plus_username_is_taken() {
        let o = decide(
            "billieeilish",
            200,
            r#"<meta property="og:title" content="Billie Eilish">{"username":"billieeilish"} /billieeilish"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn missing_404_is_available() {
        let o = decide("x", 404, "nope", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn not_found_view_is_available_on_200_and_404() {
        // REGRESSÃO (research 2026-10): view "We can't find that user."
        // caía em ambiguous-200 => RateLimited, escondendo nick livre.
        for s in [200u16, 404u16] {
            let o = decide("x", s, "<h1>We can't find that user.</h1>", 5);
            assert_eq!(o.result, CheckResult::Available, "status {s}");
        }
    }

    #[test]
    fn shell_without_markers_is_never_decided() {
        let o = decide("x", 200, "<title>SoundCloud</title>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
