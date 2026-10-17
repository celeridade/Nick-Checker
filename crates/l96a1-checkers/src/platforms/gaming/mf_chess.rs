//! Chess (Sherlock manifest): https://www.chess.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: gaming | via: keyword | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfChess;

pub const URL: &str = "https://www.chess.com/member/{}";
pub const URL_MAIN: &str = "https://www.chess.com/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Username is valid"],
    error_code: &[],
    regex: Some("^[a-zA-Z0-9_]{3,25}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://www.chess.com/callback/user/valid?username={}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfChess {
    fn key(&self) -> &'static str {
        "mf:chess"
    }
    fn name(&self) -> &'static str {
        "Chess"
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
