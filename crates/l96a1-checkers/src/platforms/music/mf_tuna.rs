//! Tuna (Sherlock manifest): https://tuna.voicemod.net/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: music | via: override | NSFW: false | sample: bob
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfTuna;

pub const URL: &str = "https://tuna.voicemod.net/user/{}";
pub const URL_MAIN: &str = "https://tuna.voicemod.net/";
pub const SAMPLE: Option<&str> = Some("bob");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[a-z0-9]{4,40}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfTuna {
    fn key(&self) -> &'static str {
        "mf:tuna"
    }
    fn name(&self) -> &'static str {
        "Tuna"
    }
    fn group(&self) -> &'static str {
        "music"
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
