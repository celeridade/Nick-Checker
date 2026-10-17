//! minds (Sherlock manifest): https://www.minds.com
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: social | via: wmn-crossref | NSFW: false | sample: john
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfMinds;

pub const URL: &str = "https://www.minds.com/{}/";
pub const URL_MAIN: &str = "https://www.minds.com";
pub const SAMPLE: Option<&str> = Some("john");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["\"valid\":true"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://www.minds.com/api/v3/register/validate?username={}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfMinds {
    fn key(&self) -> &'static str {
        "mf:minds"
    }
    fn name(&self) -> &'static str {
        "minds"
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
