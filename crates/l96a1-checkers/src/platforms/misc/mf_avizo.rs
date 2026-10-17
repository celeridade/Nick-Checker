//! Avizo (Sherlock manifest): https://www.avizo.cz/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: misc | via: fallback | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfAvizo;

pub const URL: &str = "https://www.avizo.cz/{}/";
pub const URL_MAIN: &str = "https://www.avizo.cz/";
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
    error_url: Some("https://www.avizo.cz/"),
};

#[async_trait]
impl l96a1_core::Platform for MfAvizo {
    fn key(&self) -> &'static str {
        "mf:avizo"
    }
    fn name(&self) -> &'static str {
        "Avizo"
    }
    fn group(&self) -> &'static str {
        "misc"
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
