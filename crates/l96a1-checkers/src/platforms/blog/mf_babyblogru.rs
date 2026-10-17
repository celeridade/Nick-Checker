//! babyblogRU (Sherlock manifest): https://www.babyblog.ru/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: blog | via: keyword | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfBabyblogru;

pub const URL: &str = "https://www.babyblog.ru/user/{}";
pub const URL_MAIN: &str = "https://www.babyblog.ru/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://www.babyblog.ru/"),
};

#[async_trait]
impl l96a1_core::Platform for MfBabyblogru {
    fn key(&self) -> &'static str {
        "mf:babyblogru"
    }
    fn name(&self) -> &'static str {
        "babyblogRU"
    }
    fn group(&self) -> &'static str {
        "blog"
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
