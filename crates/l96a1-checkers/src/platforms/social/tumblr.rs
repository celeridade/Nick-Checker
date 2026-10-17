//! Tumblr: lowercase canonico; decide no contraste taken/avail.
//! Research (`research/tumblr.md`): nossa rede toma 403 de perímetro
//! (direto e proxies) — rota residencial/sticky ou Unknown honesto.
//! Via residencial: taken = blog (`@nick / nick.tumblr.com` + posts);
//! avail = landing ("Join over 100 million").

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct Tumblr;

/// Decisão pura da page.
fn decide(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    let mut out = if status == 404 && lower.contains("not found") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "404-title",
            )],
        )
    } else if status == 200
        && (body.contains(&format!("@{u} /")) || body.contains(&format!("{u}.tumblr.com")))
    {
        // Self-ref de blog + posts = taken (2 sinais).
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "blog-selfref"),
                l96a1_core::Observation::taken_strong("web", "tumblr-domain"),
            ],
        )
    } else if status == 200 && lower.contains("join over 100 million") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "web",
                "landing",
            )],
        )
    } else if status == 200 && (body.contains("\"blog\":") || body.contains("login_required")) {
        // Só vale em 200: parede 403/consent com `login_required` NÃO é
        // prova de blog (audit 2026-10 — sem o gate virava falso Taken).
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "web",
                "blog-or-login",
            )],
        )
    } else {
        // 403 de perímetro / consent-shell / página irreconhecível: NÃO é
        // prova de throttling => Unknown honesto (nunca RateLimited).
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Tumblr {
    fn key(&self) -> &'static str {
        "tumblr"
    }
    fn name(&self) -> &'static str {
        "Tumblr"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.tumblr.com/{}", user.to_lowercase())
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(1, 32, r"^[a-z0-9-]+$")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let lower = user.to_lowercase();
        let url = format!("https://www.tumblr.com/{lower}");
        match common::get(ctx, &url, &[]).await {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => decide(&lower, s, &b, ms()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn blog_selfref_is_taken() {
        let o = decide("staff", 200, "@staff / staff.tumblr.com posts", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn landing_is_available() {
        let o = decide("zzqxkvw123nonexist", 200, "Join over 100 million people", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn not_found_404_is_available() {
        let o = decide("x", 404, "<title>Not found.</title>", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn wall_with_login_required_is_never_taken() {
        // REGRESSÃO (audit 2026-10): 403 de perímetro com
        // `login_required` caía no braço blog-or-login => falso Taken.
        let o = decide("staff", 403, r#"{"login_required":true}"#, 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }
}
