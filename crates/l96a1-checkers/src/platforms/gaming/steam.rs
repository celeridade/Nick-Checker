//! Steam: /id/{u} — `g_rgProfileData` = taken, "could not be found" = avail.
//! Research (`research/steam.md`): par confirmado (57 KB vs 27 KB).
//! Nota: `/id/` é vanity URL (conta sem vanity = outro formato).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Steam;

/// Decisão pura da page.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if body.contains("g_rgProfileData =") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "profile-data"),
                l96a1_core::Observation::taken_strong("web", "profile-data2"),
            ],
        )
    } else if body.contains("could not be found") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::avail_authoritative("web", "not-found"),
                l96a1_core::Observation::avail_strong("web", "not-found2"),
            ],
        )
    } else {
        common::finish_http(
            t,
            status,
            body,
            &["g_rgProfileData ="],
            &["could not be found"],
        )
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Steam {
    fn key(&self) -> &'static str {
        "steam"
    }
    fn name(&self) -> &'static str {
        "Steam"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://steamcommunity.com/id/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://steamcommunity.com/id/{user}");
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
    fn profile_data_is_taken() {
        let o = decide(200, "var g_rgProfileData = {};", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn not_found_text_is_available() {
        let o = decide(200, "The specified profile could not be found.", 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
