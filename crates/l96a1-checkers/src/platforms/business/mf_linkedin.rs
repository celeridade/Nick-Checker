//! LinkedIn (Sherlock manifest): https://linkedin.com
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: business | via: override | NSFW: false | sample: paulpfeister
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfLinkedin;

pub const URL: &str = "https://linkedin.com/in/{}";
pub const URL_MAIN: &str = "https://linkedin.com";
pub const SAMPLE: Option<&str> = Some("paulpfeister");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[a-zA-Z0-9]{3,100}$"),
    method: "GET",
    payload: None,
    headers: &[("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"), ("Accept-Language", "en-US,en;q=0.9"), ("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfLinkedin {
    fn key(&self) -> &'static str {
        "mf:linkedin"
    }
    fn name(&self) -> &'static str {
        "LinkedIn"
    }
    fn group(&self) -> &'static str {
        "business"
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
