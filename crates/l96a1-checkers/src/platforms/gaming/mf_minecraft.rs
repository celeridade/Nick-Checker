//! Minecraft (Sherlock manifest): https://minecraft.net/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: gaming | via: native-inherit | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfMinecraft;

pub const URL: &str = "https://api.mojang.com/users/profiles/minecraft/{}";
pub const URL_MAIN: &str = "https://minecraft.net/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Couldn't find any profile with name"],
    error_code: &[],
    regex: Some("^.{1,25}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfMinecraft {
    fn key(&self) -> &'static str {
        "mf:minecraft"
    }
    fn name(&self) -> &'static str {
        "Minecraft"
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
