//! Discord (Sherlock manifest): https://discord.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: gaming | via: native-inherit | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfDiscord;

pub const URL: &str = "https://discord.com";
pub const URL_MAIN: &str = "https://discord.com/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["{\"taken\":false}"],
    error_code: &[],
    regex: None,
    method: "POST",
    payload: Some("{\"username\": \"{}\"}"),
    headers: &[("Content-Type", "application/json")],
    url_probe: Some("https://discord.com/api/v9/unique-username/username-attempt-unauthed"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfDiscord {
    fn key(&self) -> &'static str {
        "mf:discord"
    }
    fn name(&self) -> &'static str {
        "Discord"
    }
    fn group(&self) -> &'static str {
        "gaming"
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
