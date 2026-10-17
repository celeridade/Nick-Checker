//! znanylekarz.pl (Sherlock manifest): https://znanylekarz.pl
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: health | via: override | NSFW: false | sample: janusz-nowak
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfZnanylekarzPl;

pub const URL: &str = "https://www.znanylekarz.pl/{}";
pub const URL_MAIN: &str = "https://znanylekarz.pl";
pub const SAMPLE: Option<&str> = Some("janusz-nowak");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfZnanylekarzPl {
    fn key(&self) -> &'static str {
        "mf:znanylekarz.pl"
    }
    fn name(&self) -> &'static str {
        "znanylekarz.pl"
    }
    fn group(&self) -> &'static str {
        "health"
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
