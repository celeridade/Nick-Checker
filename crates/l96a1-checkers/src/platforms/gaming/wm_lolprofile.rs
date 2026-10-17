//! LoLProfile (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: bea, wild
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmLolprofile;

pub const URI: &str = "https://lolprofile.net/search/world/{account}-world";
pub const SAMPLE: Option<&str> = Some("bea");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"content sw\">",
    m_code: 200,
    m_string: "We could not find any results, please try again later or check your input.",
};

#[async_trait]
impl l96a1_core::Platform for WmLolprofile {
    fn key(&self) -> &'static str {
        "wm:lolprofile"
    }
    fn name(&self) -> &'static str {
        "LoLProfile"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 24, needs_proxy: false, sticky_proxy: false }
    }
    fn profile_url(&self, user: &str) -> String {
        URI.replace("{account}", user)
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(1, 100, r"^.*$")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        site_probe::probe_wmn(&BAKED, URI, user, ctx).await
    }
}
