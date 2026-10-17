//! Freelancer (Sherlock manifest): https://www.freelancer.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: business | via: wmn-crossref | NSFW: false | sample: red0xff
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfFreelancer;

pub const URL: &str = "https://www.freelancer.com/u/{}";
pub const URL_MAIN: &str = "https://www.freelancer.com/";
pub const SAMPLE: Option<&str> = Some("red0xff");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["\"users\":{}"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://www.freelancer.com/api/users/0.1/users?usernames%5B%5D={}&compact=true"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfFreelancer {
    fn key(&self) -> &'static str {
        "mf:freelancer"
    }
    fn name(&self) -> &'static str {
        "Freelancer"
    }
    fn group(&self) -> &'static str {
        "business"
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
