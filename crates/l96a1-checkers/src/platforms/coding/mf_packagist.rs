//! Packagist (Sherlock manifest): https://packagist.org/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: coding | via: override | NSFW: false | sample: psr
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfPackagist;

pub const URL: &str = "https://packagist.org/packages/{}/";
pub const URL_MAIN: &str = "https://packagist.org/";
pub const SAMPLE: Option<&str> = Some("psr");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://packagist.org/search/?q={}&reason=vendor_not_found"),
};

#[async_trait]
impl l96a1_core::Platform for MfPackagist {
    fn key(&self) -> &'static str {
        "mf:packagist"
    }
    fn name(&self) -> &'static str {
        "Packagist"
    }
    fn group(&self) -> &'static str {
        "coding"
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
