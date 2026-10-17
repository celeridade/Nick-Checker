//! YouTube: handle @ — canon exato=taken, 404-título=avail.
//! Research (`research/youtube.md`, sessão 2026-10-10): `canonicalBaseUrl`
//! é o campo real de handle (não existe `"channelHandle"` no HTML).
//! Taken exige `200` + `canonicalBaseUrl:"/@<nick>"` exato (case-insensitive,
//! YT ignora case: `/@Amma`) + `channelId`. Só `channelId` NÃO basta:
//! paredes/busca/consentimento trazem IDs alheios (falsos taken).
//! Caveat: canal TERMINADO pode virar 404 — follow-up com oEmbed como 2º
//! sinal p/ nicks de valor. Caveat 2: `@` livre + `/user/` legado ocupado
//! (ex.: `aamm/abba/akka`) = claim BLOQUEADO no Studio
//! (`CHANNEL_HANDLE_VALIDATION_RESULT_UNAVAILABLE`) — por isso o checker
//! sonda o namespace legado como 2º sinal: `/user/` com `joinedDateText`
//! é taken (nick existe, embeds normais), nunca Unknown.
//! Regra Blackbird/WMN ("YouTube User2", verificada 2026-10 nesta rede):
//! handle existente => 303 -> 200 + `canonicalBaseUrl`/`channelId`;
//! handle livre => 404 + `<title>404 Not Found`. Sem marcador => Unknown.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Youtube;

/// Decisão pura da page do handle.
fn decide(status: u16, body: &str, user: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    // Dono exato da página (ex.: `"canonicalBaseUrl":"/@mkbhd"`).
    let canon_exact = lower.contains(&format!("\"canonicalbaseurl\":\"/@{u}\""));
    let has_channel = lower.contains("\"channelid\"");
    let mut out = if status == 200 && canon_exact && has_channel {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "handle",
                "canon-exact",
            )],
        )
    } else if status == 404
        && (lower.contains("<title>404 not found")
            || lower.contains("this channel does not exist"))
    {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "handle",
                "404-title",
            )],
        )
    } else if status == 404 || status == 200 {
        // 200 sem canon exato (parede, busca, redirect p/ outro canal,
        // consentimento) ou 404 sem marcador: Unknown honesto, nunca taken.
        common::finish(t, vec![])
    } else {
        common::finish_http(
            t,
            status,
            body,
            &["\"channelId\"", "canonicalBaseUrl"],
            &["<title>404 Not Found", "This channel does not exist"],
        )
    };
    out.latency_ms = ms;
    out
}

/// Decisão pura do namespace legado `/user/` (2º sinal).
/// `Some(taken)` se o nick existe como custom URL legada (ex.: `/user/aamm`
/// = "Andrew Amm", `/user/abba` = "ABBA"); `None` caso contrário — o caller
/// mantém o veredito do `@`.
fn decide_legacy(status: u16, body: &str, ms: u64) -> Option<CheckOutcome> {
    let t = Instant::now();
    if status == 200 && body.contains("joinedDateText") {
        let mut out = common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "legacy",
                "user-namespace",
            )],
        );
        out.latency_ms = ms;
        Some(out)
    } else {
        None
    }
}

#[async_trait]
impl l96a1_core::Platform for Youtube {
    fn key(&self) -> &'static str {
        "youtube"
    }
    fn name(&self) -> &'static str {
        "YouTube"
    }
    fn group(&self) -> &'static str {
        "streaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.youtube.com/@{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 64, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // Sinal 1: namespace `@` (canon exato).
        let url = format!("https://www.youtube.com/@{user}");
        let first = match common::get(ctx, &url, &[]).await {
            None => return common::outcome_net_err(ms()),
            Some((s, b)) => decide(s, &b, user, ms()),
        };
        if first.result == l96a1_core::CheckResult::Taken {
            return first;
        }
        // Sinal 2: namespace legado `/user/` (+`/about`, onde mora o
        // `joinedDateText`) — nick existente aqui bloqueia o claim do `@`
        // (Studio: UNAVAILABLE), então é taken, nunca Unknown.
        let legacy_url = format!("https://www.youtube.com/user/{user}/about");
        match common::get(ctx, &legacy_url, &[]).await {
            Some((s, b)) => {
                if let Some(o) = decide_legacy(s, &b, ms()) {
                    return o;
                }
                first
            }
            None => first,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn channel_id_is_taken() {
        let o = decide(
            200,
            r#"{"channelId":"UCxxxx","canonicalBaseUrl":"/@xqda"}"#,
            "xqda",
            5,
        );
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn canon_match_is_case_insensitive() {
        // YT ignora case no handle (`/@Amma` dona de `amma`).
        let o = decide(
            200,
            r#"{"channelId":"UCyyyy","canonicalBaseUrl":"/@Amma"}"#,
            "amma",
            5,
        );
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn stray_channel_id_without_canon_is_unknown() {
        // Regressão 2026-10-10 (falsos taken): 200 com `channelId` alheio
        // (parede/busca/consentimento) e canon de OUTRO canal nunca é taken.
        let o = decide(
            200,
            r#"{"channelId":"UCzzzz","canonicalBaseUrl":"/@someoneelse"}"#,
            "aamm",
            5,
        );
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }

    #[test]
    fn missing_404_title_is_available() {
        // Regra Blackbird/WMN "YouTube User2": 404 + título = livre.
        let o = decide(404, "<html><head><title>404 Not Found</title></head></html>", "aamm", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn markerless_404_is_unknown() {
        // 404 sem marcador (parede/error genérico) não prova ausência.
        let o = decide(404, "<html><body>error</body></html>", "aamm", 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn legacy_user_namespace_is_taken() {
        // `/user/aamm` = "Andrew Amm": nick existe no legado = taken.
        let o = decide_legacy(200, "<html>...joinedDateText...</html>", 5);
        let o = o.expect("legado existente deve decidir taken");
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn legacy_missing_keeps_prior_verdict() {
        // Legado 404 não decide nada: caller mantém o veredito do `@`.
        assert!(decide_legacy(404, "<title>404 Not Found</title>", 5).is_none());
        assert!(decide_legacy(200, "<title>Before you continue to YouTube</title>", 5).is_none());
    }
}
