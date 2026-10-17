//! ArtStation (Sherlock manifest): https://www.artstation.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: art | via: wmn-crossref | NSFW: false | sample: Blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfArtstation;

pub const URL: &str = "https://www.artstation.com/{}";
pub const URL_MAIN: &str = "https://www.artstation.com/";
pub const SAMPLE: Option<&str> = Some("Blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://www.artstation.com/users/{}.json"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfArtstation {
    fn key(&self) -> &'static str {
        "mf:artstation"
    }
    fn name(&self) -> &'static str {
        "ArtStation"
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
