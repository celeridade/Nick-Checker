//! CodeSandbox (Sherlock manifest): https://codesandbox.io
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: coding | via: wmn-crossref | NSFW: false | sample: icyjoseph
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfCodesandbox;

pub const URL: &str = "https://codesandbox.io/u/{}";
pub const URL_MAIN: &str = "https://codesandbox.io";
pub const SAMPLE: Option<&str> = Some("icyjoseph");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Could not find user with username"],
    error_code: &[],
    regex: Some("^[a-zA-Z0-9_-]{3,30}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://codesandbox.io/api/v1/users/{}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfCodesandbox {
    fn key(&self) -> &'static str {
        "mf:codesandbox"
    }
    fn name(&self) -> &'static str {
        "CodeSandbox"
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
