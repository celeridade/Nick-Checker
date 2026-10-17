//! Bluesky (Sherlock manifest): https://bsky.app/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: social | via: native-inherit | NSFW: false | sample: mcuban
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfBluesky;

pub const URL: &str = "https://bsky.app/profile/{}.bsky.social";
pub const URL_MAIN: &str = "https://bsky.app/";
pub const SAMPLE: Option<&str> = Some("mcuban");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor={}.bsky.social"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfBluesky {
    fn key(&self) -> &'static str {
        "mf:bluesky"
    }
    fn name(&self) -> &'static str {
        "Bluesky"
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
