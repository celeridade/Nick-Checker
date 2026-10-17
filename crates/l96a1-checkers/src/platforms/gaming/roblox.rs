//! Roblox: auth validate API — code==0 disponivel.
//! Research (`research/roblox.md`): `code:1 "already in use"` vs
//! `code:0 "valid"`. Outros codes (conteúdo impróprio etc.) também são
//! indisponíveis — evidência registra o código p/ sniper fino.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct Roblox;

/// Decisão pura da API (só o `code` decide).
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 200 {
        let code = serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("code").and_then(|c| c.as_i64()));
        match code {
            Some(0) => common::finish(
                t,
                vec![l96a1_core::Observation::avail_authoritative(
                    "api",
                    "code:0",
                )],
            ),
            // Evidência distingue taken-real (1) de outros códigos
            // (ex.: conteúdo impróprio) — sniper fino lê o evidence.
            Some(1) => common::finish(
                t,
                vec![l96a1_core::Observation::taken_authoritative(
                    "api",
                    "code:1-in-use",
                )],
            ),
            Some(_) => common::finish(
                t,
                vec![l96a1_core::Observation::taken_authoritative(
                    "api",
                    "code:nonzero",
                )],
            ),
            None => common::finish_http(t, 200, body, &[], &[]),
        }
    } else {
        common::finish_http(t, status, body, &[], &[])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Roblox {
    fn key(&self) -> &'static str {
        "roblox"
    }
    fn name(&self) -> &'static str {
        "Roblox"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.roblox.com/users/profile?username={user}")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(3, 20, r"^[a-z0-9_]+$")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!(
            "https://auth.roblox.com/v1/usernames/validate?username={user}&birthday=2000-01-01T00:00:00.000Z"
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
    fn code_zero_is_available() {
        let o = decide(200, r#"{"code":0,"message":"Username is valid"}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn code_nonzero_is_taken_with_code_in_evidence() {
        let o = decide(200, r#"{"code":1,"message":"Username is already in use"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
        assert!(o.evidence.contains("code:1-in-use"));
        let o = decide(200, r#"{"code":2,"message":"Username is inappropriate"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }
}
