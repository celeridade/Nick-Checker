//! SoylentNews (Sherlock manifest): https://soylentnews.org
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: news | via: keyword | NSFW: false | sample: adam
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfSoylentnews;

pub const URL: &str = "https://soylentnews.org/~{}";
pub const URL_MAIN: &str = "https://soylentnews.org";
pub const SAMPLE: Option<&str> = Some("adam");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["The user you requested does not exist, no matter how much you wish this might be the case."],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfSoylentnews {
    fn key(&self) -> &'static str {
        "mf:soylentnews"
    }
    fn name(&self) -> &'static str {
        "SoylentNews"
    }
    fn group(&self) -> &'static str {
        "news"
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
