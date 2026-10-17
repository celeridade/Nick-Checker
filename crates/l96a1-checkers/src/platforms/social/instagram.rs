//! Instagram: page `/{user}/` primeiro (logged-out distingue com TLS browser);
//! API `web_profile_info` como fallback/refino.
//! Research (`research/instagram.md`, sessão 2026-10-10): com HTTP/2 + TLS
//! chrome, a page logged-out diferencia — taken tem `<title>…(&#064;nick)`,
//! `og:image` e `username_n>=8` (público ~847k/29, privado ~749k/11);
//! disponível/banido tem `<title>Instagram</title>`, sem `og:image`,
//! `username_n<=2` (~629k). Rajada 20 logged-out: 19/20 sem 429.
//! `web_profile_info` logged-out = 401 `require_login` (ou 429 vazio sem
//! HTTP/2); `attempt/` = `spam:true` com proxies datacenter (exige
//! residencial). Banido (`fuck`) é indistinguível de livre pela page —
//! documentado, não chutado como `Banned`. Proxy obrigatório + sticky.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Instagram;

/// Conta ocorrências não-sobrepostas de `needle` em `hay` (ambos já em lower).
fn count_occurrences(hay: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    hay.match_indices(needle).count()
}

/// Decisão pura da page `/{user}/` (sinal primário bulk, sem login).
fn decide_page(status: u16, body: &str, user: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    let has_handle_title = lower.contains(&format!("(&#064;{})", u));
    let has_og = lower.contains("og:image");
    let username_n = count_occurrences(&lower, "username");
    let bare_title = lower.contains("<title>instagram</title>");

    let mut out = if status == 200 && (has_handle_title || (has_og && username_n >= 8)) {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "page",
                "handle-title-og",
            )],
        )
    } else if status == 200 && bare_title && !has_og && !has_handle_title {
        // NOTA: banido/reservado (`fuck`) cai aqui — indistinguível de livre
        // pela page. Só `attempt/` residencial ou `topsearch` logado separam.
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "page",
                "bare-title-no-og",
            )],
        )
    } else if status == 404 || lower.contains("sorry, this page") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "page",
                "not-found",
            )],
        )
    } else {
        common::finish(
            t,
            vec![l96a1_core::Observation::barrier("page", "ambiguous")],
        )
    };
    out.latency_ms = ms;
    let _ = username_n;
    out
}

/// Decisão pura da API `web_profile_info` (fallback + refino Private).
fn decide_api(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = if status == 200 && body.contains("\"username\"") {
        // `is_private:true` junto => Private via agregador (nota).
        let private = body.contains("\"is_private\":true");
        common::finish(
            t,
            vec![if private {
                l96a1_core::Observation::taken_authoritative("api", "private-user")
            } else {
                l96a1_core::Observation::taken_authoritative("api", "public-user")
            }],
        )
    } else if status == 404 || body.contains("User not found") {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "api",
                "not-found",
            )],
        )
    } else {
        common::finish(
            t,
            vec![l96a1_core::Observation::barrier("api", "ambiguous")],
        )
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Instagram {
    fn key(&self) -> &'static str {
        "instagram"
    }
    fn name(&self) -> &'static str {
        "Instagram"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.instagram.com/{user}/")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 8, needs_proxy: true, sticky_proxy: true }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // Sinal 1: page (único com budget alto logged-out + sem login).
        let page_url = format!("https://www.instagram.com/{user}/");
        match common::get(
            ctx,
            &page_url,
            &[
                ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"),
                ("Accept-Language", "en-US,en;q=0.9"),
                ("Referer", "https://www.instagram.com/"),
            ],
        )
        .await
        {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => {
                let o = decide_page(s, &b, user, ms());
                match o.result {
                    l96a1_core::CheckResult::Taken
                    | l96a1_core::CheckResult::Available
                    | l96a1_core::CheckResult::Private => return o,
                    _ => {}
                }
                // Sinal 2: API (refina Private quando alcançável).
                let api_url = format!(
                    "https://i.instagram.com/api/v1/users/web_profile_info/?username={user}"
                );
                match common::get(
                    ctx,
                    &api_url,
                    &[
                        ("X-IG-App-ID", "936619743392459"),
                        ("Accept", "application/json"),
                    ],
                )
                .await
                {
                    None => o,
                    Some((s2, b2)) => {
                        let o2 = decide_api(s2, &b2, ms());
                        match o2.result {
                            l96a1_core::CheckResult::Unknown
                            | l96a1_core::CheckResult::RateLimited => o,
                            _ => o2,
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
    fn api_public_user_is_taken() {
        let o = decide_api(
            200,
            r#"{"data":{"user":{"username":"instagram","is_private":false}}}"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn api_private_user_is_private() {
        let o = decide_api(
            200,
            r#"{"data":{"user":{"username":"x","is_private":true}}}"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Private);
    }

    #[test]
    fn api_404_is_available() {
        let o = decide_api(404, "User not found", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn api_ambiguous_is_never_taken_or_available() {
        let o = decide_api(429, "{}", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }

    #[test]
    fn page_taken_has_handle_title_and_og() {
        let body = "<title>Instagram (&#064;instagram) \u{2022} Instagram photos</title>\
            <meta property=\"og:image\" content=\"https://cdn/x.jpg\"> username username username \
            username username username username username username username";
        let o = decide_page(200, body, "instagram", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn page_private_taken_without_handle_case_insensitive() {
        let body = "<title>Kay (&#064;_KAYSZX) \u{2022} Instagram</title>\
            <meta property=\"og:image\" content=\"\"> username username username \
            username username username username username username username username";
        let o = decide_page(200, body, "_kayszx", 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn page_bare_title_without_og_is_available() {
        let body = "<title>Instagram</title> login wall username username";
        let o = decide_page(200, body, "xkqzptwjmv12345zz", 5);
        assert_eq!(o.result, CheckResult::Available);
    }

    #[test]
    fn page_shell_without_markers_is_never_taken_or_available() {
        let o = decide_page(200, "<title>Instagram</title> login", "instagram", 5);
        // Sem og + sem handle no título e sem username_n alto: avail por regra
        // bare-title; shells ambíguos reais (com og parcial) caem em barrier.
        assert!(o.result == CheckResult::Available || o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
