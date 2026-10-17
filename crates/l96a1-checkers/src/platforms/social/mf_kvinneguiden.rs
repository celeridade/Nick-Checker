//! Kvinneguiden (Sherlock manifest): https://forum.kvinneguiden.no
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: social | via: keyword | NSFW: false | sample: blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfKvinneguiden;

pub const URL: &str = "https://forum.kvinneguiden.no";
pub const URL_MAIN: &str = "https://forum.kvinneguiden.no";
pub const SAMPLE: Option<&str> = Some("blue");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["{\"result\":\"ok\"}"],
    error_code: &[],
    regex: Some("^[a-zA-Z0-9_.-]{3,18}$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: Some("https://forum.kvinneguiden.no/?app=core&module=system&controller=ajax&do=usernameExists&input={}"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfKvinneguiden {
    fn key(&self) -> &'static str {
        "mf:kvinneguiden"
    }
    fn name(&self) -> &'static str {
        "Kvinneguiden"
    }
    fn group(&self) -> &'static str {
        "social"
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
