//! Linktree (Sherlock manifest): https://linktr.ee/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: social | via: wmn-crossref | NSFW: false | sample: anne
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfLinktree;

pub const URL: &str = "https://linktr.ee/{}";
pub const URL_MAIN: &str = "https://linktr.ee/";
pub const SAMPLE: Option<&str> = Some("anne");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["\"statusCode\":404"],
    error_code: &[],
    regex: Some("^[\\w\\.]{2,30}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfLinktree {
    fn key(&self) -> &'static str {
        "mf:linktree"
    }
    fn name(&self) -> &'static str {
        "Linktree"
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
