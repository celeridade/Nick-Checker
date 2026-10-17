//! Tenor (Sherlock manifest): https://tenor.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: images | via: wmn-crossref | NSFW: false | sample: red
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfTenor;

pub const URL: &str = "https://tenor.com/users/{}";
pub const URL_MAIN: &str = "https://tenor.com/";
pub const SAMPLE: Option<&str> = Some("red");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[A-Za-z0-9_]{2,32}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfTenor {
    fn key(&self) -> &'static str {
        "mf:tenor"
    }
    fn name(&self) -> &'static str {
        "Tenor"
    }
    fn group(&self) -> &'static str {
        "images"
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
