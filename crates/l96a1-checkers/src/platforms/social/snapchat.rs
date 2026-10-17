//! Snapchat: `/add/` 308 => `/@user` — decide no destino FINAL.
//! Research (`research/snapchat.md`): marcador antigo `snapchat.com/add/`
//! MORTO (ausente na page) → ocupado virava RateLimited.
//! Taken: 200 + `displayName`/JSON de perfil; avail: 404.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Snapchat;

/// Decisão pura da page final (`/@user`).
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 404 {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "404",
            )],
        )
    } else if status == 200 && body.contains("displayName") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "displayName"),
                l96a1_core::Observation::taken_strong("web", "profile-json"),
            ],
        )
    } else {
        common::finish(t, vec![l96a1_core::Observation::barrier("web", "ambiguous")])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Snapchat {
    fn key(&self) -> &'static str {
        "snapchat"
    }
    fn name(&self) -> &'static str {
        "Snapchat"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.snapchat.com/add/{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 8, needs_proxy: true, sticky_proxy: true }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // reqwest segue o 308 até /@user sozinho; decide no destino.
        let url = format!("https://www.snapchat.com/add/{user}");
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
    fn taken_has_display_name() {
        let o = decide(200, r#"{"displayName":"Team Snapchat"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn avail_404() {
        let o = decide(404, "nope", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn old_add_marker_absence_is_not_taken() {
        // REGRESSÃO: marcador antigo morto gerava RateLimited em taken.
        // Sem displayName, 200 ambíguo nunca é Taken/Available.
        let o = decide(200, "<html>generic</html>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
