//! TryHackMe (Sherlock manifest): https://tryhackme.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: tech | via: wmn-crossref | NSFW: false | sample: ashu
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfTryhackme;

pub const URL: &str = "https://tryhackme.com/p/{}";
pub const URL_MAIN: &str = "https://tryhackme.com/";
pub const SAMPLE: Option<&str> = Some("ashu");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["{\"success\":false}"],
    error_code: &[],
    regex: Some("^[a-zA-Z0-9.]{1,16}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://tryhackme.com/api/user/exist/{}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfTryhackme {
    fn key(&self) -> &'static str {
        "mf:tryhackme"
    }
    fn name(&self) -> &'static str {
        "TryHackMe"
    }
    fn group(&self) -> &'static str {
        "tech"
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
