//! TikTok: sessões logadas (árbitro) + oembed (leve) + page (`privateAccount`).
//! Ordem (espelho Discord: pomelo authed => unauthed => suggestions):
//! 1. pool de sessões (`tiktok_sessions.txt`) via `GET /api/uniqueid/check/`
//!    — veredito real do servidor vira autoritativo e mata falsos
//!    positivos Available; sem veredito => fallthrough silencioso;
//! 2. oembed 200+`author_url` = Taken (400 é AMBÍGUO: privado ou livre);
//! 3. page: `"privateAccount":true` + self-ref = Private; bloco
//!    `webapp.user-detail` com status != 0 e sem `"userInfo":{` = Available;
//!    shell de bot/consent (SEM o bloco user-detail) = Unknown honesto.
//!
//! ANTI-RATE (research 2026-10, verificado com 100 proxies + IP direto):
//! - oembed e page vivem em BUCKETS INDEPENDENTES (oembed 429 direto
//!   enquanto a page segue 200 no mesmo IP). Teto oembed >24 reqs
//!   sequenciais no mesmo proxy sem 429 — o bucket é saudável via proxy.
//! - A causa do rate em bulk era `sticky_proxy: true` (94 nicks x até
//!   2 reqs todos no MESMO proxy = ~150 reqs em rajada num IP só).
//!   Com rotação (`sticky_proxy: false`) cada IP recebe ~1-2 reqs.
//! - 429 / `ratelimit triggered` / `overload-protect` / 403 NUNCA caem
//!   para a page (seria queimar 370KB no mesmo IP morto e ainda
//!   mascarar o rate como Unknown): retornam `RateLimited` direto para
//!   a engine retentar em OUTRO proxy.
//! - ~20% dos proxies datacenter recebem shell de bot/consent (200 sem
//!   `userInfo`) em vez da page completa: sem self-ref => Unknown
//!   honesto, nunca Available/Private (regra ouro).
//! - Page só decide com SELF-REF (`"uniqueId":"{nick}"` do perfil
//!   consultado). `privateAccount` solto aparece no bundle JS.
//! HARDENING (2026-10, verificado em browser real): a page só decide com
//! SELF-REF (`"uniqueId":"{nick}"` do perfil consultado). Shells de
//! bot/consent (sem userInfo) e ocorrências espúrias de `privateAccount`
//! fora do bloco do usuário => Unknown, nunca Available/Private.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Tiktok;

/// Headers de browser real (só UA derruba a taxa de page completa:
/// ~20% dos proxies recebem shell sem eles; oembed também agradece).
const OEMBED_HEADERS: &[(&str, &str)] = &[
    ("Accept", "application/json, text/plain, */*"),
    ("Accept-Language", "en-US,en;q=0.9"),
    ("Referer", "https://www.tiktok.com/"),
];

const PAGE_HEADERS: &[(&str, &str)] = &[
    (
        "Accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
    ),
    ("Accept-Language", "en-US,en;q=0.9"),
    ("Referer", "https://www.tiktok.com/"),
    ("Upgrade-Insecure-Requests", "1"),
    ("Sec-Fetch-Dest", "document"),
    ("Sec-Fetch-Mode", "navigate"),
    ("Sec-Fetch-Site", "same-origin"),
];

/// Barreira de rate/WAF: 429, `ratelimit triggered` (oembed e page),
/// `overload-protect` (embed/edge) ou 403 (IP flagado). Tudo isso pede
/// OUTRO proxy — nunca cair para o próximo sinal no mesmo IP.
fn is_barrier(status: u16, body: &str) -> bool {
    if matches!(status, 429 | 403 | 503) {
        return true;
    }
    let lower = body.to_lowercase();
    lower.contains("ratelimit triggered")
        || lower.contains("overload-protect")
        || lower.contains("overload triggered")
}

/// Decisão pura do oembed. 400 NÃO decide (ambíguo: privado ou livre).
fn decide_oembed(status: u16, body: &str, ms: u64) -> Result<CheckOutcome, ()> {
    let mut out = match (status, body) {
        (200, b) if b.contains("author_url") => common::finish(
            Instant::now(),
            vec![l96a1_core::Observation::taken_authoritative("oembed", "author_url")],
        ),
        (404, _) => common::finish(
            Instant::now(),
            vec![l96a1_core::Observation::avail_authoritative("oembed", "404")],
        ),
        _ => return Err(()),
    };
    out.latency_ms = ms;
    Ok(out)
}

/// Decisão pura da page (2º sinal, só chamado no caminho ambíguo).
/// ATENÇÃO: `privateAccount` solto aparece no bundle JS — só vale a
/// forma dado `"privateAccount":true` DENTRO do perfil consultado
/// (self-ref `"uniqueId":"{user}"`).
///
/// Sem self-ref há DOIS casos distintos (research 2026-10, 3 nicks livres
/// + shell comparados lado a lado):
/// - page COMPLETA de nick livre: bloco `webapp.user-detail` presente com
///   `statusCode` != 0 (observado 10221) e SEM `"userInfo":{` => Available;
/// - shell de bot/consent (proxy datacenter flagado): SEM o bloco
///   `webapp.user-detail` => Unknown honesto (indistinguível).
/// Textos tipo "couldn't find this account" NÃO servem de marcador: estão
/// no bundle i18n e aparecem até na page de perfil TAKEN.
/// CAVEAT: conta banida/suspensa tende ao mesmo bloco 10221 sem userInfo
/// (TikTok não expõe estado banido distinto) — pode voltar Available e
/// falhar só no claim. Documentado, sem ramo `Banned`.
fn detail_has_no_user(user_detail_block: &str, full_body: &str) -> bool {
    // Bloco presente + status != 0 + sem userInfo em LUGAR NENHUM.
    let code_ok = user_detail_block
        .strip_prefix("\"statusCode\":")
        .and_then(|rest| {
            let digits: String =
                rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse::<i64>().ok()
        })
        .map(|c| c != 0)
        .unwrap_or(false);
    code_ok && !full_body.contains("\"userInfo\":{")
}

fn decide_page(user: &str, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let self_ref =
        lower.contains(&format!("\"uniqueid\":\"{}\"", user.to_lowercase()));
    let mut out = if !self_ref {
        // Sem self-ref: livre (bloco detail sem user) ou shell (sem bloco).
        let free = body
            .find("webapp.user-detail\"")
            .and_then(|i| {
                // Pega `{"statusCode":NNN,...}` logo após o nome do bloco.
                let after = body[i + "webapp.user-detail\"".len()..].trim_start();
                after
                    .strip_prefix(':')
                    .map(|b| b.trim_start().strip_prefix('{').unwrap_or(b))
            })
            .map(|inner| detail_has_no_user(inner, body))
            .unwrap_or(false);
        if free {
            common::finish(
                t,
                vec![l96a1_core::Observation::avail_authoritative(
                    "page",
                    "detail-no-user",
                )],
            )
        } else {
            // Shell de bot/consent ou challenge: nada do perfil => Unknown.
            common::finish(t, vec![])
        }
    } else if body.contains("\"privateAccount\":true") {
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "page",
                "private:true",
            )],
        )
    } else if body.contains("\"userInfo\":{") {
        // NOTA: evidência sem o substring "private" (se não vira Private).
        // Forma com `{`: só o objeto real do bloco user-detail decide
        // (strings soltas "userInfo" no bundle JS não valem).
        common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative(
                "page",
                "public-user",
            )],
        )
    } else {
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Tiktok {
    fn key(&self) -> &'static str {
        "tiktok"
    }
    fn name(&self) -> &'static str {
        "TikTok"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.tiktok.com/@{user}")
    }
    fn validator(&self) -> l96a1_core::UsernameValidator {
        // Regra oficial do cliente (user.<hash>.js): `eR=/^[0-9a-z._A-Z]{2,24}$/`,
        // `<2` min2char, `>24` max. Normalize() já lowercases antes do match.
        l96a1_core::UsernameValidator::new(2, 24, r"^[a-z0-9_.]{2,24}$")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        // needs_proxy: oembed queima o IP direto em poucas reqs (429
        // persistente); via pool de 100 o teto é >24/IP sem rate.
        // sticky_proxy: false de propósito — sticky concentrava ~150
        // reqs de um bulk de 94 nicks num IP só (o 1/94 de rate).
        // Rotativo = ~1-2 reqs/IP = zero rate.
        l96a1_core::PlatformProfile { max_concurrent: 12, needs_proxy: true, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        // Método 0 (sessões logadas, espelho do pomelo Discord): árbitro
        // definitivo quando há veredito real; sem sessões o custo é zero
        // (nem entra). Rate autenticado NÃO retorna direto: o unauthed
        // ainda pode decidir (ex.: Taken público) — o rate só vale se o
        // fallback for inconclusivo (backoff honesto p/ a engine).
        let mut authed_rate: Option<CheckOutcome> = None;
        if !ctx.tiktok_sessions.is_empty() {
            if let Some(mut o) =
                super::tiktok_session::attempt_pool(user, ctx, ms()).await
            {
                o.latency_ms = ms();
                match o.result {
                    l96a1_core::CheckResult::Taken | l96a1_core::CheckResult::Available => {
                        return o;
                    }
                    l96a1_core::CheckResult::RateLimited => authed_rate = Some(o),
                    _ => {}
                }
            }
        }
        let unauthed = Self::check_unauthed(user, ctx, t).await;
        match unauthed.result {
            l96a1_core::CheckResult::Unknown => authed_rate.unwrap_or(unauthed),
            _ => unauthed,
        }
    }
}

impl Tiktok {
    /// Cascata unauthed (oembed leve + page): caminho de sempre, idêntico
    /// com ou sem sessões configuradas.
    async fn check_unauthed(user: &str, ctx: &CheckCtx, t: Instant) -> CheckOutcome {
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://www.tiktok.com/oembed?url=https://www.tiktok.com/@{user}");
        let (status, body) = match common::get(ctx, &url, OEMBED_HEADERS).await {
            None => return common::outcome_net_err(ms()),
            Some((s, b)) => (s, b),
        };
        // Barreira no oembed: retorna RateLimited SEM queimar a page no
        // mesmo IP morto (a engine retenta em outro proxy).
        if is_barrier(status, &body) {
            let mut o = common::finish(
                t,
                vec![l96a1_core::Observation::barrier("oembed", "rate-limited")],
            );
            o.latency_ms = ms();
            return o;
        }
        // 200/404 decidem; resto (incl. 400 ambíguo) => page.
        if status == 200 || status == 404 {
            if let Ok(o) = decide_oembed(status, &body, ms()) {
                return o;
            }
        }
        let page_url = format!("https://www.tiktok.com/@{user}");
        match common::get(ctx, &page_url, PAGE_HEADERS).await {
            None => common::outcome_net_err(ms()),
            Some((ps, pb)) => {
                if is_barrier(ps, &pb) {
                    let mut o = common::finish(
                        t,
                        vec![l96a1_core::Observation::barrier("page", "rate-limited")],
                    );
                    o.latency_ms = ms();
                    return o;
                }
                decide_page(user, &pb, ms())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;
    use l96a1_core::Platform;

    #[test]
    fn oembed_200_author_url_is_taken() {
        let o = decide_oembed(200, r#"{"author_url":"https://www.tiktok.com/@x"}"#, 5).unwrap();
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn oembed_400_is_ambiguous_never_available() {
        // REGRESSÃO: 400 já foi classificado como avail (falso positivo
        // em TODA conta privada). Agora não decide.
        assert!(decide_oembed(400, r#"{"message":"Something went wrong","code":400}"#, 5).is_err());
    }

    #[test]
    fn barrier_markers_never_fall_through() {
        // REGRESSÃO (bulk 1/94): 429 caía para a page no mesmo IP morto
        // (370KB queimados) e o shell voltava Unknown, escondendo o
        // rate da engine — sem retry em outro proxy.
        assert!(is_barrier(429, "ratelimit triggered"));
        assert!(is_barrier(200, "ratelimit triggered"));
        assert!(is_barrier(503, "overload-protect triggered"));
        assert!(is_barrier(403, "<html>blocked</html>"));
        assert!(!is_barrier(200, r#"{"author_url":"https://www.tiktok.com/@x"}"#));
        assert!(!is_barrier(400, r#"{"message":"Something went wrong","code":400}"#));
        assert!(!is_barrier(200, "<html>Couldn't find this account</html>"));
    }

    #[test]
    fn validator_enforces_official_2_24_charset() {
        // Regex+limites oficiais do bundle (eR + min2char/max).
        let v = Tiktok.validator();
        assert!(v.normalize("quilometragem").is_some());
        assert!(v.normalize("ab").is_some());
        assert!(v.normalize("a").is_none());
        assert!(v.normalize("x".repeat(24).as_str()).is_some());
        assert!(v.normalize("x".repeat(25).as_str()).is_none());
        assert!(v.normalize("bad-char!").is_none());
        assert!(v.normalize("UPPER").is_some()); // normalize lowercases
    }

    #[test]
    fn profile_rotates_proxy_never_sticky() {
        // REGRESSÃO: sticky concentrava o bulk inteiro num IP só.
        let p = Tiktok.profile();
        assert!(p.needs_proxy);
        assert!(!p.sticky_proxy);
    }

    #[test]
    fn page_private_marker_is_private() {
        let o = decide_page(
            "hikaru",
            r#"..."uniqueId":"hikaru",... "privateAccount":true ...}"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Private);
    }

    #[test]
    fn page_without_selfref_is_unknown_never_available() {
        // REGRESSÃO: shell de bot/consent (sem userInfo) virava falso HIT.
        let o = decide_page("x", "<html>Couldn't find this account</html>", 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn page_free_detail_without_userinfo_is_available() {
        // Bloco real de nick livre (3 amostras live, statusCode 10221):
        // user-detail presente + status != 0 + sem "userInfo":{ => HIT.
        let body = r#"...<script id="__UNIVERSAL_DATA_FOR_REHYDRATION__" type="application/json">{"__DEFAULT_SCOPE__":{"webapp.user-detail":{"statusCode":10221,"statusMsg":"","needFix":false},"webapp.a-b":{"b_c":"2"}}}</script>..."#;
        let o = decide_page("zzqxkvw123nonexist999", body, 5);
        assert_eq!(o.result, CheckResult::Available);
        assert!(o.evidence.contains("detail-no-user"));
    }

    #[test]
    fn page_shell_without_detail_block_is_unknown() {
        // Shell de proxy flagado: universal-data SEM webapp.user-detail
        // (taken e livre geram o mesmo shell) => Unknown honesto.
        let body = r#"...<script id="__UNIVERSAL_DATA_FOR_REHYDRATION__" type="application/json">{"__DEFAULT_SCOPE__":{"webapp.app-context":{"language":"en"},"webapp.a-b":{"b_c":"0"}}}</script>..."#;
        let o = decide_page("qxzzfree01", body, 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn page_detail_status_zero_without_userinfo_is_unknown() {
        // Guarda contra backend instável: bloco sem user mas status 0
        // (lookup não concluiu?) nunca vira HIT.
        let body = r#"...{"webapp.user-detail":{"statusCode":0,"statusMsg":"","needFix":false}}..."#;
        let o = decide_page("x", body, 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn page_private_without_selfref_is_unknown() {
        // `privateAccount:true` espúrio fora do bloco do usuário não decide.
        let o = decide_page("x", r#"..."uniqueId":"other",... "privateAccount":true ...}"#, 5);
        assert_eq!(o.result, CheckResult::Unknown);
    }

    #[test]
    fn page_public_user_is_taken() {
        let o = decide_page(
            "x",
            r#"..."uniqueId":"x",... "userInfo":{... "privateAccount":false ...}"#,
            5,
        );
        assert_eq!(o.result, CheckResult::Taken);
    }
}
