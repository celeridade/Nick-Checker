//! Chatujme.cz (Sherlock manifest): https://chatujme.cz/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: social | via: keyword | NSFW: false | sample: david
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfChatujmeCz;

pub const URL: &str = "https://profil.chatujme.cz/{}";
pub const URL_MAIN: &str = "https://chatujme.cz/";
pub const SAMPLE: Option<&str> = Some("david");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Neexistujicí profil"],
    error_code: &[],
    regex: Some("^[a-zA-Z][a-zA-Z1-9_-]*$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfChatujmeCz {
    fn key(&self) -> &'static str {
        "mf:chatujme.cz"
    }
    fn name(&self) -> &'static str {
        "Chatujme.cz"
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
