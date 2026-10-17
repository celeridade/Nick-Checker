//! ResearchGate (Sherlock manifest): https://www.researchgate.net/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: hobby | via: wmn-crossref | NSFW: false | sample: John_Smith
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfResearchgate;

pub const URL: &str = "https://www.researchgate.net/profile/{}";
pub const URL_MAIN: &str = "https://www.researchgate.net/";
pub const SAMPLE: Option<&str> = Some("John_Smith");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: Some("\\w+_\\w+"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://www.researchgate.net/directory/profiles"),
};

#[async_trait]
impl l96a1_core::Platform for MfResearchgate {
    fn key(&self) -> &'static str {
        "mf:researchgate"
    }
    fn name(&self) -> &'static str {
        "ResearchGate"
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
