//! Archive.org (Sherlock manifest): https://archive.org
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: hobby | via: override | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfArchiveOrg;

pub const URL: &str = "https://archive.org/details/@{}";
pub const URL_MAIN: &str = "https://archive.org";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["could not fetch an account with user item identifier", "The resource could not be found", "Internet Archive services are temporarily offline"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://archive.org/details/@{}?noscript=true"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfArchiveOrg {
    fn key(&self) -> &'static str {
        "mf:archive.org"
    }
    fn name(&self) -> &'static str {
        "Archive.org"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
