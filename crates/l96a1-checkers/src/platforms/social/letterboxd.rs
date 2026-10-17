//! Letterboxd: endpoint de registro `/s/checkusername?q=` (decodificado
//! do `bxd.initUsernameChecker`). Taxonomia: AVAILABLE / TAKEN / INVALID
//! (`a-z 0-9 _`, sem ponto) / ILLEGAL ("Unavailable", bloqueadas=taken) /
//! TOO_SHORT (<2) / TOO_LONG / MISSING / YOU. Input `maxlength=15`.
//! Via datacenter cai no challenge CF => Unknown. Nunca assumir por nome
//! (`@letterboxd` NÃO existe).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct Letterboxd;

/// Decisão pura do endpoint. `q` já validado pelo caller quando preciso.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 200 && body.contains("\"result\":\"AVAILABLE\"") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "signup-api",
                "AVAILABLE",
            )],
        )
    } else if body.contains("\"result\":\"TAKEN\"") || body.contains("\"result\":\"ILLEGAL\"") {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("signup-api", "taken"),
                l96a1_core::Observation::taken_strong("signup-api", "taken2"),
            ],
        )
    } else if body.contains("\"result\":\"INVALID\"")
        || body.contains("\"result\":\"TOO_SHORT\"")
        || body.contains("\"result\":\"TOO_LONG\"")
        || body.contains("\"result\":\"MISSING\"")
    {
        // Formato inválido: Unknown honesto (validator local já filtra; nunca avail).
        common::finish(t, vec![])
    } else {
        // Challenge CF / 404 / shell sem taxonomia: Unknown honesto.
        // (Só a taxonomia AVAILABLE/TAKEN do endpoint decide.)
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Letterboxd {
    fn key(&self) -> &'static str {
        "letterboxd"
    }
    fn name(&self) -> &'static str {
        "Letterboxd"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://letterboxd.com/{user}/")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(2, 15, r"^[a-z0-9_]+$")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // Sinal 1: endpoint de registro (resposta JSON taxonomizada).
        let url = format!("https://letterboxd.com/s/checkusername?q={user}");
        match common::get(ctx, &url, &[("X-Requested-With", "XMLHttpRequest")]).await {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => {
                let o = decide(s, &b, ms());
                match o.result {
                    l96a1_core::CheckResult::Taken | l96a1_core::CheckResult::Available => return o,
                    _ => {}
                }
                // Sinal 2: page direta (profile-header).
                let purl = format!("https://letterboxd.com/{user}/");
                match common::get(ctx, &purl, &[]).await {
                    None => o,
                    Some((ps, pb)) => {
                        if ps == 200 && pb.contains("profile-header") {
                            let mut po = common::finish(
                                Instant::now(),
                                vec![
                                    l96a1_core::Observation::taken_strong("web", "profile-header"),
                                    l96a1_core::Observation::taken_strong("web", "profile-header2"),
                                ],
                            );
                            po.latency_ms = ms();
                            po
                        } else {
                            o
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn available_result_is_available() {
        let o = decide(200, r#"{"result":true,"data":{"result":"AVAILABLE"}}"#, 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn taken_and_illegal_are_taken() {
        let o = decide(200, r#"{"data":{"result":"TAKEN"}}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
        let o = decide(200, r#"{"data":{"result":"ILLEGAL"}}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn invalid_format_is_never_available() {
        for r in ["INVALID", "TOO_SHORT", "TOO_LONG", "MISSING"] {
            let o = decide(200, &format!(r#"{{"data":{{"result":"{r}"}}}}"#), 5);
            assert!(
                o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited,
                "{r}"
            );
        }
    }

    #[test]
    fn challenge_wall_is_never_available() {
        let o = decide(403, "<title>Just a moment...</title>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
