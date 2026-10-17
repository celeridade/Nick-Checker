//! LushStories (Sherlock manifest): https://www.lushstories.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: xx NSFW xx | via: nsfw-flag | NSFW: true | sample: chris_brown
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfLushstories;

pub const URL: &str = "https://www.lushstories.com/profile/{}";
pub const URL_MAIN: &str = "https://www.lushstories.com/";
pub const SAMPLE: Option<&str> = Some("chris_brown");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://www.lushstories.com/login"),
};

#[async_trait]
impl l96a1_core::Platform for MfLushstories {
    fn key(&self) -> &'static str {
        "mf:lushstories"
    }
    fn name(&self) -> &'static str {
        "LushStories"
    }
    fn group(&self) -> &'static str {
        "xx NSFW xx"
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
