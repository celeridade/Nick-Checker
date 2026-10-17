//! LibraryThing (Sherlock manifest): https://www.librarything.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: hobby | via: wmn-crossref | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfLibrarything;

pub const URL: &str = "https://www.librarything.com/profile/{}";
pub const URL_MAIN: &str = "https://www.librarything.com/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["<p>Error: This user doesn't exist</p>"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[("Cookie", "LTAnonSessionID=3159599315; LTUnifiedCookie=%7B%22areyouhuman%22%3A1%7D; ")],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfLibrarything {
    fn key(&self) -> &'static str {
        "mf:librarything"
    }
    fn name(&self) -> &'static str {
        "LibraryThing"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
