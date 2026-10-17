//! GitLab: /api/v4/users?username= — array vazio=avail.
//! Research (`research/gitlab.md`): confirmado. Gancho `Banned`: o array
//! traz `state` (`active` visto) — `state != active` => Banned.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Gitlab;

/// Decisão pura da API.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 200 && body.trim() == "[]" {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "[]")],
        )
    } else if status == 200 && body.contains("\"id\"") {
        // state != active (banned/blocked) => Banned, não Taken.
        let banned = body.contains("\"state\":\"banned\"")
            || body.contains("\"state\":\"blocked\"")
            || body.contains("\"locked\":true");
        common::finish(
            t,
            vec![if banned {
                l96a1_core::Observation::banned_authoritative("api", "state")
            } else {
                l96a1_core::Observation::taken_authoritative("api", "id")
            }],
        )
    } else {
        common::finish_http(t, status, body, &["\"id\""], &[])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Gitlab {
    fn key(&self) -> &'static str {
        "gitlab"
    }
    fn name(&self) -> &'static str {
        "GitLab"
    }
    fn group(&self) -> &'static str {
        "dev"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://gitlab.com/{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://gitlab.com/api/v4/users?username={user}");
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
    fn empty_array_is_available() {
        assert_eq!(decide(200, "[]", 5).result, CheckResult::Available);
    }

    #[test]
    fn active_user_is_taken() {
        let o = decide(200, r#"[{"id":781,"username":"root","state":"active","locked":false}]"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn non_active_state_is_banned() {
        let o = decide(200, r#"[{"id":1,"username":"x","state":"banned","locked":true}]"#, 5);
        assert_eq!(o.result, CheckResult::Banned);
    }
}
