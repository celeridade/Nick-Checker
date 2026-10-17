//! TETR.IO (Sherlock manifest): https://tetr.io
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: gaming | via: wmn-crossref | NSFW: false | sample: osk
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfTetrIo;

pub const URL: &str = "https://ch.tetr.io/u/{}";
pub const URL_MAIN: &str = "https://tetr.io";
pub const SAMPLE: Option<&str> = Some("osk");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["No such user!"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://ch.tetr.io/api/users/{}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfTetrIo {
    fn key(&self) -> &'static str {
        "mf:tetr.io"
    }
    fn name(&self) -> &'static str {
        "TETR.IO"
    }
    fn group(&self) -> &'static str {
        "gaming"
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
