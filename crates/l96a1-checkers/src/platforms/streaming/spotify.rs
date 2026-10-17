//! Spotify: page SEM SINAL (shell idêntico nos estados).
//! Research (`research/spotify.md`): `spotify:user:` e JSON ausentes até
//! no taken; oEmbed não suporta perfil (400 nos dois). Sem chute.
//! Caminho real: Web API com app token (`client_credentials`, sem login)
//! — precisa `client_id`/`client_secret` (padrão dos tokens do projeto).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Spotify;

/// Decisão pura da page. Shell sozinho NUNCA decide (sem sinal).
/// Herdada por `wm:spotify`.
pub(crate) fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 404 {
        // 404 real (se um dia vier) = avail. Shell 200 genérico = nada.
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("web", "404")],
        )
    } else if body.contains("spotify:user:") && body.contains("\"username\"") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "uri"),
                l96a1_core::Observation::taken_strong("web", "uri2"),
            ],
        )
    } else {
        // Shell idêntico nos dois estados (verificado): sem sinal => Unknown.
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Spotify {
    fn key(&self) -> &'static str {
        "spotify"
    }
    fn name(&self) -> &'static str {
        "Spotify"
    }
    fn group(&self) -> &'static str {
        "streaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://open.spotify.com/user/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://open.spotify.com/user/{user}");
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
    fn generic_shell_is_never_decided() {
        // REGRESSÃO: shell idêntico não pode virar Taken nem Available.
        let o = decide(200, "<title>Spotify - Web Player</title>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }

    #[test]
    fn real_404_is_available() {
        let o = decide(404, "nope", 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
