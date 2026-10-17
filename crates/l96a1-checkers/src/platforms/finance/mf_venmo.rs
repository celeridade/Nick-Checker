//! Venmo (Sherlock manifest): https://venmo.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: finance | via: wmn-crossref | NSFW: false | sample: jenny
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfVenmo;

pub const URL: &str = "https://account.venmo.com/u/{}";
pub const URL_MAIN: &str = "https://venmo.com/";
pub const SAMPLE: Option<&str> = Some("jenny");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Venmo | Page Not Found"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[("Host", "account.venmo.com")],
    url_probe: Some("https://test1.venmo.com/u/{}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfVenmo {
    fn key(&self) -> &'static str {
        "mf:venmo"
    }
    fn name(&self) -> &'static str {
        "Venmo"
    }
    fn group(&self) -> &'static str {
        "finance"
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
