//! HackenProof (Hackers) (Sherlock manifest): https://hackenproof.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: coding | via: keyword | NSFW: false | sample: blazezaria
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfHackenproofHackers;

pub const URL: &str = "https://hackenproof.com/hackers/{}";
pub const URL_MAIN: &str = "https://hackenproof.com/";
pub const SAMPLE: Option<&str> = Some("blazezaria");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Page not found"],
    error_code: &[],
    regex: Some("^[\\w-]{,34}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfHackenproofHackers {
    fn key(&self) -> &'static str {
        "mf:hackenproof-(hackers)"
    }
    fn name(&self) -> &'static str {
        "HackenProof (Hackers)"
    }
    fn group(&self) -> &'static str {
        "coding"
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
