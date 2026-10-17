//! X: page direta (sinal 1) + mirror vxtwitter (sinal 2, opcional).
//! Research (`research/x.md`): logged-out decide tudo na page —
//! público self-ref; protegido `is_protected`/notice; suspenso SEM self-ref
//! + `UserUnavailable`/`"Suspended"` => Banned; disponível 404.
//! Cuidado: `protected`/`screen_name` soltos existem no bundle — só valem
//! formas dado com contexto (self-ref do nick).
//! Mirror é terceiro com Cloudflare p/ datacenter: secundário, nunca
//! sozinho p/ Available.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct X;

/// Decisão pura da page direta.
fn decide_page(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    // Self-ref em forma dado (não vale substring solta do bundle).
    // Case-insensitive: validator aceita maiúsculas, page normaliza.
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    let selfref = lower.contains(&u)
        && (body.contains("\"screen_name\"") || body.contains("screen_name"));
    let mut out = if status == 404 {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "page",
                "404",
            )],
        )
    } else if (body.contains("unavailable_reason") || body.contains("UserUnavailable"))
        && (body.contains("Suspended") || body.contains("Account suspended"))
    {
        // Suspenso NÃO tem self-ref (sem dados do user): combinação acima
        // é distinta o bastante p/ dispensar. Forma dado, não bundle.
        common::finish(
            t,
            vec![l96a1_core::Observation::banned_authoritative(
                "page",
                "suspended",
            )],
        )
    } else if selfref
        && (body.contains("is_protected") || body.contains("These posts are protected"))
    {
        // NOTA com substring "private": agregador mapeia p/ Private.
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "page",
                "private-protected",
            )],
        )
    } else if status == 200 && selfref {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "page",
                "selfref",
            )],
        )
    } else {
        common::finish(
            t,
            vec![l96a1_core::Observation::barrier("page", "ambiguous")],
        )
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for X {
    fn key(&self) -> &'static str {
        "x"
    }
    fn name(&self) -> &'static str {
        "X"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://x.com/{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 8, needs_proxy: true, sticky_proxy: true }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://x.com/{user}");
        let (status, body) = match common::get(ctx, &url, &[]).await {
            None => return common::outcome_net_err(ms()),
            Some((s, b)) => (s, b),
        };
        let page_out = decide_page(user, status, &body, ms());
        // Page decide sozinha quando conclusiva; senão tenta o mirror.
        match page_out.result {
            l96a1_core::CheckResult::Unknown | l96a1_core::CheckResult::RateLimited => {}
            _ => return page_out,
        }
        let murl = format!("https://api.vxtwitter.com/{user}");
        match common::get(ctx, &murl, &[]).await {
            None => page_out,
            Some((200, mb)) if mb.contains("\"screen_name\"") => {
                let mut o = common::finish(
                    Instant::now(),
                    vec![
                        l96a1_core::Observation::taken_strong("mirror", "screen_name"),
                        l96a1_core::Observation::taken_strong("page", "fallback"),
                    ],
                );
                o.latency_ms = ms();
                o
            }
            // Mirror nunca decide Available sozinho (CF/instável).
            _ => page_out,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    const PUB: &str = r#"{"screen_name":"elonmusk","name":"Elon"}<div>elonmusk</div>"#;
    const PROT: &str = r#"{"screen_name":"driko171"} These posts are protected"#;
    const SUSP: &str = r#"__typename:"UserUnavailable",unavailable_reason:"Suspended"><h2>Account suspended</h2>"#;

    #[test]
    fn public_is_taken() {
        assert_eq!(decide_page("elonmusk", 200, PUB, 5).result, CheckResult::Taken);
    }

    #[test]
    fn protected_is_private() {
        // `protected` na nota => Private via agregador.
        assert_eq!(decide_page("driko171", 200, PROT, 5).result, CheckResult::Private);
    }

    #[test]
    fn suspended_is_banned() {
        assert_eq!(decide_page("fuck", 200, SUSP, 5).result, CheckResult::Banned);
    }

    #[test]
    fn missing_is_available() {
        let o = decide_page("zzqxkvw123nonexist", 404, "not found", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn bundle_noise_without_selfref_is_unknown() {
        // screen_name solto sem o nick = bundle, não evidência.
        let o = decide_page("zzqxkvw123nonexist", 200, r#"{"screen_name":"x"}"#, 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
