//! NationStates Nation (Sherlock manifest): https://nationstates.net
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: gaming | via: override | NSFW: false | sample: the_holy_principality_of_saint_mark
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfNationstatesNation;

pub const URL: &str = "https://nationstates.net/nation={}";
pub const URL_MAIN: &str = "https://nationstates.net";
pub const SAMPLE: Option<&str> = Some("the_holy_principality_of_saint_mark");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["Was this your nation? It may have ceased to exist due to inactivity, but can rise again!"],
    error_code: &[],
    regex: None,
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfNationstatesNation {
    fn key(&self) -> &'static str {
        "mf:nationstates-nation"
    }
    fn name(&self) -> &'static str {
        "NationStates Nation"
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
