//! Codeforces (Sherlock manifest): https://codeforces.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: coding | via: wmn-crossref | NSFW: false | sample: tourist
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfCodeforces;

pub const URL: &str = "https://codeforces.com/profile/{}";
pub const URL_MAIN: &str = "https://codeforces.com/";
pub const SAMPLE: Option<&str> = Some("tourist");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::StatusCode,
    error_msg: &[],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://codeforces.com/api/user.info?handles={}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfCodeforces {
    fn key(&self) -> &'static str {
        "mf:codeforces"
    }
    fn name(&self) -> &'static str {
        "Codeforces"
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
