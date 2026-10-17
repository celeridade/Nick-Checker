//! LessWrong (Sherlock manifest): https://www.lesswrong.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: social | via: override | NSFW: false | sample: habryka
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfLesswrong;

pub const URL: &str = "https://www.lesswrong.com/users/{}";
pub const URL_MAIN: &str = "https://www.lesswrong.com/";
pub const SAMPLE: Option<&str> = Some("habryka");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://www.lesswrong.com/"),
};

#[async_trait]
impl l96a1_core::Platform for MfLesswrong {
    fn key(&self) -> &'static str {
        "mf:lesswrong"
    }
    fn name(&self) -> &'static str {
        "LessWrong"
    }
    fn group(&self) -> &'static str {
        "social"
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
