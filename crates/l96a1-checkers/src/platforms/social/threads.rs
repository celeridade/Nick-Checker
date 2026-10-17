//! Threads: API primeiro, page `threads.com` como 2º sinal.
//! Research (`research/threads.md`): API dá 429 nesta rota (muro igual ao
//! IG); `threads.net` 301 => `threads.com` (seguir!); shells opacos.
//! Sem sinal provado => Unknown (nunca chute).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Threads;

/// Decisão pura da API.
fn decide_api(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    // Self-match: `"username"` solto aparece em shells de erro — só vale
    // com o nick consultado no valor.
    let u = user.to_lowercase();
    let lower = body.to_lowercase();
    let selfname = lower.contains(&format!("\"username\":\"{u}\""))
        || lower.contains(&format!("\"username\": \"{u}\""));
    let mut out = if status == 200 && selfname {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "api",
                "username-selfref",
            )],
        )
    } else if status == 404 {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "404")],
        )
    } else {
        // 400/403/shell sem self-ref: Unknown honesto (nunca RateLimited).
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

/// Decisão pura da page `threads.com` (shell opaco: só user-data conta).
/// `has_user` = JSON de usuário com self-ref (a calibrar em fixtures).
fn decide_page(has_user: bool, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if has_user {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_strong(
                "page",
                "user-json",
            )],
        )
    } else {
        // Shell sozinho não prova ausência nem throttling: Unknown honesto.
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Threads {
    fn key(&self) -> &'static str {
        "threads"
    }
    fn name(&self) -> &'static str {
        "Threads"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.threads.net/@{user}")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 8, needs_proxy: true, sticky_proxy: true }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://www.threads.net/api/v1/users/web_profile_info/?username={user}");
        let api_out = match common::get(
            ctx,
            &url,
            &[
                ("X-IG-App-ID", "936619743392459"),
                ("Accept", "application/json"),
                ("X-Requested-With", "XMLHttpRequest"),
            ],
        )
        .await
        {
            None => return common::outcome_net_err(ms()),
            Some((s, b)) => decide_api(user, s, &b, ms()),
        };
        match api_out.result {
            l96a1_core::CheckResult::Taken | l96a1_core::CheckResult::Available => return api_out,
            _ => {}
        }
        // 2º sinal: page direta (threads.com após 301).
        let purl = format!("https://www.threads.com/@{user}");
        match common::get(ctx, &purl, &[]).await {
            None => api_out,
            Some((_, pb)) => {
                let lower = pb.to_lowercase();
                let has_user =
                    lower.contains("\"username\"") && lower.contains(&user.to_lowercase());
                let po = decide_page(has_user, ms());
                match po.result {
                    l96a1_core::CheckResult::Taken => po,
                    // Page nunca declara Available sozinha (shell opaco).
                    _ => api_out,
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
    fn api_username_is_taken() {
        let o = decide_api("zuck", 200, r#"{"data":{"user":{"username":"zuck"}}}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn api_username_without_selfref_is_unknown() {
        // REGRESSÃO: `"username"` solto em shell de erro virava Taken/rate.
        let o = decide_api("zuck", 200, r#"{"error":"username lookup failed"}"#, 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn api_404_is_available() {
        let o = decide_api("x", 404, "{}", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn page_shell_alone_never_frees() {
        // REGRESSÃO: shell opaco não pode virar Available.
        let o = decide_page(false, 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
