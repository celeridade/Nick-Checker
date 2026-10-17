//! RubyGems (Sherlock manifest): https://rubygems.org/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: coding | via: override | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfRubygems;

pub const URL: &str = "https://rubygems.org/profiles/{}";
pub const URL_MAIN: &str = "https://rubygems.org/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[a-zA-Z][a-zA-Z0-9_-]{1,40}"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfRubygems {
    fn key(&self) -> &'static str {
        "mf:rubygems"
    }
    fn name(&self) -> &'static str {
        "RubyGems"
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
