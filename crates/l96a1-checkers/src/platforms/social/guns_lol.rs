//! guns.lol: self-redirect 307 + cookie-jar MANUAL (sem ele o reqwest
//! entra em loop e tudo vira Error). Taken = perfil (avatar/links) sem
//! not-found; avail = not-found (`not found`/`everything you want`).
//! Alias premium (borda . _ -) = barreira (nunca disponível normal).

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct GunsLol;

/// Decisão pura da page FINAL (após redirects + cookies).
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    // Claim page ("Username not found") vem em 200/404 (verificado ao
    // vivo 2026-10). Fora disso, "not found" pode ser texto de parede
    // (CF/5xx) — nunca Available (audit 2026-10).
    let not_found = lower.contains("not found")
        || lower.contains("everything you want")
        || lower.contains("bulunamad");
    let mut out = if (status == 200 || status == 404) && not_found {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::avail_authoritative("web", "not-found"),
                l96a1_core::Observation::avail_strong("web", "not-found2"),
            ],
        )
    } else if status == 200
        && (lower.contains("avatar") || lower.contains("/links") || lower.contains("profile"))
    {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("web", "profile"),
                l96a1_core::Observation::taken_strong("web", "profile2"),
            ],
        )
    } else {
        common::finish(t, vec![l96a1_core::Observation::barrier("web", "ambiguous")])
    };
    out.latency_ms = ms;
    out
}

/// Resolve Location relativa contra a URL base.
fn join_location(base: &str, loc: &str) -> String {
    if loc.starts_with("http://") || loc.starts_with("https://") {
        return loc.to_string();
    }
    let root = base
        .splitn(4, '/')
        .take(3)
        .collect::<Vec<_>>()
        .join("/");
    if loc.starts_with('/') {
        format!("{root}{loc}")
    } else {
        let prefix = base.rsplit_once('/').map(|(p, _)| p).unwrap_or(base);
        format!("{prefix}/{loc}")
    }
}

#[async_trait]
impl l96a1_core::Platform for GunsLol {
    fn key(&self) -> &'static str {
        "gunslol"
    }
    fn name(&self) -> &'static str {
        "guns.lol"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://guns.lol/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // Alias premium: comeca/termina com . _ - => pulado (nunca disponivel normal).
        if user.starts_with(['.', '_', '-']) || user.ends_with(['.', '_', '-']) {
            return common::finish(
                t,
                vec![l96a1_core::Observation::barrier("rule", "premium-alias")],
            );
        }
        let url = format!("https://guns.lol/{user}");
        // Passo 1: sem seguir (captura set-cookies do self-redirect).
        let (status, _stub, cookies) = match common::get_nofollow(ctx, &url, &[]).await {
            None => return common::outcome_net_err(ms()),
            Some(v) => v,
        };
        if status == 200 {
            // Sem redirect: decide direto (raro, mas barato).
            let body = match common::get(ctx, &url, &[]).await {
                None => return common::outcome_net_err(ms()),
                Some((_, b)) => b,
            };
            return decide(200, &body, ms());
        }
        // Passo 2: segue com cookie-jar em client SEM follow automático
        // (o follow nativo derruba os cookies entre hops).
        let client = std::sync::Arc::new(l96a1_core::build_client_nofollow(
            ctx.timeout,
            ctx.proxy_url.as_deref(),
        ));
        let mut target = url.clone();
        let mut jar = cookies.join("; ");
        for _ in 0..5 {
            let resp = client
                .get(&target)
                .header("Cookie", jar.clone())
                .send()
                .await;
            let r = match resp {
                Ok(r) => r,
                Err(_) => return common::outcome_net_err(ms()),
            };
            for v in r.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(s) = v.to_str() {
                    let c = s.split(';').next().unwrap_or("");
                    if !c.is_empty() {
                        if !jar.is_empty() {
                            jar.push_str("; ");
                        }
                        jar.push_str(c);
                    }
                }
            }
            let st = r.status().as_u16();
            // Redirect manual: Location pode vir como header ou no corpo stub.
            let loc: Option<String> = r
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            let tx = r.text().await.unwrap_or_default();
            if !(300..400).contains(&st) {
                return decide(st, &tx, ms());
            }
            let next = loc.or_else(|| {
                ["href=\"", "href='"]
                    .iter()
                    .filter_map(|p| {
                        tx.find(p).map(|i| {
                            let rest = &tx[i + p.len()..];
                            rest.split(['"', '\'']).next().unwrap_or("").to_string()
                        })
                    })
                    .next()
            });
            match next {
                Some(l) if !l.is_empty() => target = join_location(&target, &l),
                _ => return decide(st, &tx, ms()),
            }
        }
        common::finish(
            t,
            vec![l96a1_core::Observation::barrier("web", "redirect-loop")],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn join_location_forms() {
        assert_eq!(
            join_location("https://guns.lol/guns", "/guns"),
            "https://guns.lol/guns"
        );
        assert_eq!(
            join_location("https://guns.lol/guns", "https://x.test/a"),
            "https://x.test/a"
        );
    }

    #[test]
    fn not_found_is_available() {
        let o = decide(200, "<h1>User not found</h1>", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn wall_text_is_never_available() {
        // REGRESSÃO (audit 2026-10): parede 403/5xx contendo "not
        // found" caía no braço de avail => falso HIT.
        let o = decide(403, "error: requested url was not found on edge", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }

    #[test]
    fn profile_without_not_found_is_taken() {
        let o = decide(200, "<img class=\"avatar\" src=\"x\"><div class=\"links\">a</div>", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }
}
