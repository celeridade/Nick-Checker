//! 1337x (Sherlock manifest): https://www.1337x.to/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: tech | via: override | NSFW: false | sample: FitGirl
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct Mf1337x;

pub const URL: &str = "https://www.1337x.to/user/{}/";
pub const URL_MAIN: &str = "https://www.1337x.to/";
pub const SAMPLE: Option<&str> = Some("FitGirl");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["<title>Error something went wrong.</title>", "<head><title>404 Not Found</title></head>"],
    error_code: &[],
    regex: Some("^[A-Za-z0-9]{4,12}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for Mf1337x {
    fn key(&self) -> &'static str {
        "mf:1337x"
    }
    fn name(&self) -> &'static str {
        "1337x"
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
