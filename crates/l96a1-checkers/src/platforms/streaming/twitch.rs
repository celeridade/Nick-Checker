//! Twitch: twitchtracker — Overview=taken, resto=Unknown.
//! Research (`research/twitch.md`): confirmado (50 KB + Overview vs 404).
//! Terceiro instável por natureza: Helix API com app token como futuro
//! (precisa client_id/secret). Banido no tracker: mapear se aparecer.
//! HARDENING (2026-10, verificado): o tracker responde o MESMO template
//! 404 ("404 Page Not Found" + nav com "Overview</a>") p/ slug rastreado
//! e não-rastreado — 404 do tracker significa "não-indexado", NÃO "nick
//! livre no Twitch". 404 nu => Unknown, nunca Available. Roadmap: Helix
//! (`/helix/users?login=`) com `TWITCH_CLIENT_ID` opcional como 2º sinal.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Twitch;

/// Decisão pura da page do tracker.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (200, b) if b.contains("Overview</a>") => common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "overview"),
                l96a1_core::Observation::taken_strong("web", "overview2"),
            ],
        ),
        // 404 do tracker = slug não-indexado (template idêntico nos dois
        // casos) — não prova ausência no Twitch => Unknown honesto.
        (404, _) => common::finish(t, vec![]),
        (s, b) => common::finish_http(t, s, b, &["Overview</a>"], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Twitch {
    fn key(&self) -> &'static str {
        "twitch"
    }
    fn name(&self) -> &'static str {
        "Twitch"
    }
    fn group(&self) -> &'static str {
        "streaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.twitch.tv/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://twitchtracker.com/{user}");
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
    fn overview_is_taken() {
        let o = decide(200, "<a>Overview</a> stats", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn untracked_404_is_unknown_never_available() {
        // REGRESSÃO: 404 do tracker virava falso HIT p/ canal existente
        // porém não-indexado (ex: slug fora do ranking).
        let o = decide(404, "404 Page Not Found ... Overview</a> ...", 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }
}
