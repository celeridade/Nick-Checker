//! Bluesky: API publica getProfile. Normaliza actor p/ checar disponível.
//! Research (`research/bluesky.md`): nick cru sem ponto dá 400
//! "Invalid AT identifier" (≠ "not found") — normalizar `nick.bsky.social`.
//! `handle.invalid` é conta REAL: nunca assumir por nome.
//! `AccountDeactivated` (candidato) => Banned.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct Bluesky;

/// Normaliza o actor: sem ponto => `nick.bsky.social`.
fn actor_for(user: &str) -> String {
    if user.contains('.') {
        user.to_string()
    } else {
        format!("{user}.bsky.social")
    }
}

/// Decisão pura do `getProfile`.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let mut out = if status == 200 && body.contains("\"did\"") {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative("api", "did")],
        )
    } else if status == 400
        && (lower.contains("unable to resolve")
            || lower.contains("not found")
            || lower.contains("profile not found"))
    {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "api",
                "unresolvable",
            )],
        )
    } else if lower.contains("accountdeactivated")
        || lower.contains("suspend")
        || lower.contains("takedown")
        || lower.contains("deactivated")
    {
        common::finish(
            t,
            vec![l96a1_core::Observation::banned_authoritative(
                "api",
                "deactivated",
            )],
        )
    } else {
        common::finish(t, vec![l96a1_core::Observation::barrier("api", "ambiguous")])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Bluesky {
    fn key(&self) -> &'static str {
        "bluesky"
    }
    fn name(&self) -> &'static str {
        "Bluesky"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://bsky.app/profile/{user}")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(1, 30, r"^[a-z0-9.-]+$")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!(
            "https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor={}",
            actor_for(user)
        );
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
    fn actor_normalization() {
        assert_eq!(actor_for("nick"), "nick.bsky.social");
        assert_eq!(actor_for("nick.bsky.social"), "nick.bsky.social");
    }

    #[test]
    fn taken_has_did() {
        let o = decide(200, r#"{"did":"did:plc:xyz","handle":"x"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn profile_not_found_is_available() {
        let o = decide(400, r#"{"error":"InvalidRequest","message":"Profile not found"}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn deactivated_is_banned() {
        let o = decide(400, r#"{"error":"AccountDeactivated"}"#, 5);
        assert_eq!(o.result, CheckResult::Banned);
    }

    #[test]
    fn invalid_identifier_alone_is_never_available() {
        // Sem normalização isso seria o erro do nick cru — Unknown, não avail.
        let o = decide(400, r#"{"message":"Invalid AT identifier"}"#, 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
