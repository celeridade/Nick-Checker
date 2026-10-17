//! Contently (Sherlock manifest): https://contently.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: business | via: override | NSFW: false | sample: jordanteicher
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfContently;

pub const URL: &str = "https://{}.contently.com/";
pub const URL_MAIN: &str = "https://contently.com/";
pub const SAMPLE: Option<&str> = Some("jordanteicher");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: Some("^[a-zA-Z][a-zA-Z0-9_-]*$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://contently.com"),
};

#[async_trait]
impl l96a1_core::Platform for MfContently {
    fn key(&self) -> &'static str {
        "mf:contently"
    }
    fn name(&self) -> &'static str {
        "Contently"
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
