//! Chess.com: GET api/pub/player/{lower} — 200+username=taken, 404=avail.
//! Research (`research/chess_com.md`): confirmado no DIRETO (proxies tomam
//! CF 403 — rota direta obrigatória). Payload rico (`title: GM`,
//! followers, status) p/ scoring de OG. Regra edge-char mantida.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct ChessCom;

/// Decisão pura da API.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (200, b) if b.contains("\"username\"") => common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "api",
                "username",
            )],
        ),
        (404, _) => common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "404")],
        ),
        (s, b) => common::finish_http(t, s, b, &["\"username\""], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for ChessCom {
    fn key(&self) -> &'static str {
        "chesscom"
    }
    fn name(&self) -> &'static str {
        "Chess.com"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.chess.com/member/{user}")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(3, 25, r"^[a-z0-9_-]+$")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let lower = user.to_lowercase();
        if !lower.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false)
            || !lower.chars().last().map(|c| c.is_alphanumeric()).unwrap_or(false)
        {
            return common::finish(t, vec![l96a1_core::Observation::barrier("rule", "edge-char")]);
        }
        let url = format!("https://api.chess.com/pub/player/{lower}");
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
    fn username_is_taken() {
        let o = decide(200, r#"{"username":"hikaru","title":"GM","followers":1411186}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn missing_404_is_available() {
        let o = decide(404, "{}", 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
