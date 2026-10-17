//! Carbonmade (Sherlock manifest): https://carbonmade.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: business | via: wmn-crossref | NSFW: false | sample: jenny
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfCarbonmade;

pub const URL: &str = "https://{}.carbonmade.com";
pub const URL_MAIN: &str = "https://carbonmade.com/";
pub const SAMPLE: Option<&str> = Some("jenny");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[\\w@-]+?$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://carbonmade.com/fourohfour?domain={}.carbonmade.com"),
};

#[async_trait]
impl l96a1_core::Platform for MfCarbonmade {
    fn key(&self) -> &'static str {
        "mf:carbonmade"
    }
    fn name(&self) -> &'static str {
        "Carbonmade"
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
