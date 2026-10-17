//! Kongregate (Sherlock manifest): https://www.kongregate.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: gaming | via: wmn-crossref | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfKongregate;

pub const URL: &str = "https://www.kongregate.com/accounts/{}";
pub const URL_MAIN: &str = "https://www.kongregate.com/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[a-zA-Z][a-zA-Z0-9_-]*$"),
    method: "GET",
    payload: None,
    headers: &[("Accept", "text/html")],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfKongregate {
    fn key(&self) -> &'static str {
        "mf:kongregate"
    }
    fn name(&self) -> &'static str {
        "Kongregate"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 24, needs_proxy: false, sticky_proxy: false }
    }
    fn profile_url(&self, user: &str) -> String {
        site_probe::interpolate(URL, user)
    }
    fn validator(&self) -> UsernameValidator {
        site_probe::validator_for(BAKED.regex)
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        site_probe::probe_manifest(&BAKED, URL, user, ctx).await
    }
}
