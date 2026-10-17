//! GitLab (Sherlock manifest): https://gitlab.com/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: dev | via: native-inherit | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfGitlab;

pub const URL: &str = "https://gitlab.com/{}";
pub const URL_MAIN: &str = "https://gitlab.com/";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["[]"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://gitlab.com/api/v4/users?username={}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfGitlab {
    fn key(&self) -> &'static str {
        "mf:gitlab"
    }
    fn name(&self) -> &'static str {
        "GitLab"
    }
    fn group(&self) -> &'static str {
        "dev"
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
