//! Google Play (Sherlock manifest): https://play.google.com
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: shopping | via: keyword | NSFW: false | sample: GitHub
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfGooglePlay;

pub const URL: &str = "https://play.google.com/store/apps/developer?id={}";
pub const URL_MAIN: &str = "https://play.google.com";
pub const SAMPLE: Option<&str> = Some("GitHub");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["the requested URL was not found on this server"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfGooglePlay {
    fn key(&self) -> &'static str {
        "mf:google-play"
    }
    fn name(&self) -> &'static str {
        "Google Play"
    }
    fn group(&self) -> &'static str {
        "shopping"
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
