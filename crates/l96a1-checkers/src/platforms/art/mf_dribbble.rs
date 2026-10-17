//! Dribbble (Sherlock manifest): https://dribbble.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: art | via: wmn-crossref | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfDribbble;

pub const URL: &str = "https://dribbble.com/{}";
pub const URL_MAIN: &str = "https://dribbble.com/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Whoops, that page is gone."],
    error_code: &[],
    regex: Some("^[a-zA-Z][a-zA-Z0-9_-]*$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfDribbble {
    fn key(&self) -> &'static str {
        "mf:dribbble"
    }
    fn name(&self) -> &'static str {
        "Dribbble"
    }
    fn group(&self) -> &'static str {
        "art"
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
