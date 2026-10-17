//! RuneScape (Sherlock manifest): https://www.runescape.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: hobby | via: wmn-crossref | NSFW: false | sample: L33
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfRunescape;

pub const URL: &str = "https://apps.runescape.com/runemetrics/app/overview/player/{}";
pub const URL_MAIN: &str = "https://www.runescape.com/";
pub const SAMPLE: Option<&str> = Some("L33");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["{\"error\":\"NO_PROFILE\",\"loggedIn\":\"false\"}"],
    error_code: &[],
    regex: Some("^(?! )[\\w -]{1,12}(?<! )$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://apps.runescape.com/runemetrics/profile/profile?user={}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfRunescape {
    fn key(&self) -> &'static str {
        "mf:runescape"
    }
    fn name(&self) -> &'static str {
        "RuneScape"
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
