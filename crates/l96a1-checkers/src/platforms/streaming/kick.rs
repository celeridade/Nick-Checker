//! Kick: channel API — slug match=taken, 404+payload=avail.
//! Research (`research/kick.md`): confirmado, SEM WAF nesta rota
//! (force_direct). Gancho `Banned`: campo `is_banned` no JSON.
//! Nota: Cloudflare barra fingerprints nao-browser em outras rotas;
//! sob WAF a API vira Unknown (roadmap: transporte com impersonation TLS).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Kick;

/// Decisão pura da API de canal.
fn decide(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 200 {
        let v: Option<serde_json::Value> = serde_json::from_str(body).ok();
        let slug_ok = v
            .as_ref()
            .and_then(|x| x.get("slug").and_then(|s| s.as_str()))
            .map(|s| s.to_lowercase() == user.to_lowercase())
            .unwrap_or(false);
        let banned = v
            .as_ref()
            .and_then(|x| x.get("is_banned").and_then(|b| b.as_bool()))
            .unwrap_or(false);
        if banned {
            common::finish(
                t,
                vec![l96a1_core::Observation::banned_authoritative(
                    "api",
                    "is_banned",
                )],
            )
        } else if slug_ok {
            common::finish(
                t,
                vec![l96a1_core::Observation::taken_authoritative("api", "slug")],
            )
        } else {
            // JSON sem slug (WAF/shell/schema novo): Unknown honesto.
            common::finish(t, vec![])
        }
    } else if status == 404 && body.contains("Channel not found") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "api",
                "not-found",
            )],
        )
    } else {
        common::finish_http(t, status, body, &[], &["Channel not found"])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Kick {
    fn key(&self) -> &'static str {
        "kick"
    }
    fn name(&self) -> &'static str {
        "Kick"
    }
    fn group(&self) -> &'static str {
        "streaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://kick.com/{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://kick.com/api/v2/channels/{user}");
        match common::get(ctx, &url, &[("Accept", "application/json")]).await {
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
    fn slug_match_is_taken() {
        let o = decide("xqc", 200, r#"{"id":1,"slug":"xqc","is_banned":false}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn banned_flag_is_banned() {
        let o = decide("x", 200, r#"{"id":2,"slug":"x","is_banned":true}"#, 5);
        assert_eq!(o.result, CheckResult::Banned);
    }

    #[test]
    fn not_found_is_available() {
        let o = decide("x", 404, r#"{"message":"Channel not found"}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
