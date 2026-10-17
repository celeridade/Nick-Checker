//! Last.fm: endpoint de validação do signup + page 404 como 2º sinal.
//! Research (`research/lastfm.md`): `POST /join/partial/validate`
//! form-encoded (+csrf) => `userName:{valid, error_messages}`.
//! UI mostra "Ok..." nos dois casos: LER O JSON, não o texto!
//! Pages/WAF: shell idêntico nos estados; firewall até em IP residencial.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Lastfm;

/// Decisão pura do endpoint validate. Escopo ESTRITO ao bloco userName
/// (outros blocos — email/senha — têm seus próprios valid/error).
fn user_block(body: &str) -> &str {
    match body.find("\"userName\"") {
        Some(i) => {
            let rest = &body[i..];
            // Fecha no próximo bloco-irmão ("email") ou fim.
            match rest.find("\"email\"") {
                Some(j) => &rest[..j],
                None => rest,
            }
        }
        None => "",
    }
}

/// Decisão pura do endpoint validate.
fn decide_validate(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let ub = user_block(body);
    let mut out = if status == 200 && ub.contains("\"valid\": true") && !ub.contains("error_messages")
    {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "signup-api",
                "valid",
            )],
        )
    } else if ub.contains("\"valid\": false")
        || ub.contains("isn't available")
        || ub.contains("isn\u{2019}t available")
    {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("signup-api", "taken"),
                l96a1_core::Observation::taken_strong("signup-api", "taken2"),
            ],
        )
    } else {
        // Ambíguo no validate: o caller tenta o sinal 2 (page) — Unknown aqui.
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

/// Extrai o token CSRF da page `/join`.
fn csrf_of(join_body: &str) -> Option<String> {
    join_body
        .find("csrfmiddlewaretoken")
        .and_then(|i| {
            let rest = &join_body[i..];
            rest.find("value=\"").map(|j| {
                let v = &rest[j + 7..];
                v.split('"').next().unwrap_or("").to_string()
            })
        })
        .filter(|s| !s.is_empty())
}

#[async_trait]
impl l96a1_core::Platform for Lastfm {
    fn key(&self) -> &'static str {
        "lastfm"
    }
    fn name(&self) -> &'static str {
        "Last.fm"
    }
    fn group(&self) -> &'static str {
        "music"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.last.fm/user/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // Sinal 1: validate do signup (precisa CSRF da /join).
        if let Some((_, jb)) = common::get(ctx, "https://www.last.fm/join", &[]).await {
            if let Some(csrf) = csrf_of(&jb) {
                let form = format!("csrfmiddlewaretoken={csrf}&userName={user}");
                let client = std::sync::Arc::new(l96a1_core::build_client(
                    ctx.timeout,
                    ctx.proxy_url.as_deref(),
                ));
                if let Ok(r) = client
                    .post("https://www.last.fm/join/partial/validate")
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header("X-Requested-With", "XMLHttpRequest")
                    .body(form)
                    .send()
                    .await
                {
                    let st = r.status().as_u16();
                    let tx = r.text().await.unwrap_or_default();
                    let o = decide_validate(st, &tx, ms());
                    match o.result {
                        l96a1_core::CheckResult::Taken
                        | l96a1_core::CheckResult::Available => return o,
                        _ => {}
                    }
                }
            }
        }
        // Sinal 2: page 404 ("Page Not Found" p/ livre, provado pelo operador).
        match common::get(ctx, &format!("https://www.last.fm/user/{user}"), &[]).await {
            None => common::outcome_net_err(ms()),
            Some((404, b)) if b.contains("Page Not Found") => common::finish(
                Instant::now(),
                vec![l96a1_core::Observation::avail_authoritative(
                    "web",
                    "404-page",
                )],
            ),
            Some((s, b)) => common::finish_http(
                t,
                s,
                &b,
                &["header-title-display-name"],
                &["Page Not Found"],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn validate_taken_has_error_message() {
        let o = decide_validate(
            200,
            r#"{"userName": {"valid": false, "error_messages": ["Sorry, this username isn't available."]}}"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn validate_available_has_no_errors() {
        // REGRESSÃO: UI mostra "Ok..." nos dois casos; só o JSON decide.
        let o = decide_validate(
            200,
            r#"{"userName": {"valid": true, "success_message": "Ok, that username can be yours!"}}"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn csrf_extraction() {
        let html = r#"<input type="hidden" name="csrfmiddlewaretoken" value="abc123">"#;
        assert_eq!(csrf_of(html).as_deref(), Some("abc123"));
        assert!(csrf_of("<html></html>").is_none());
    }
}
