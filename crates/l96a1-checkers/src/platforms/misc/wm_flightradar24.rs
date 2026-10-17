//! Flightradar24 (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: finn, pavelkral
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFlightradar24;

pub const URI: &str = "https://my.flightradar24.com/{account}/";
pub const SAMPLE: Option<&str> = Some("finn");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"profile-card\" data-profile-user=",
    m_code: 404,
    m_string: "class=\"main page-not-found-main",
};

#[async_trait]
impl l96a1_core::Platform for WmFlightradar24 {
    fn key(&self) -> &'static str {
        "wm:flightradar24"
    }
    fn name(&self) -> &'static str {
        "Flightradar24"
    }
    fn group(&self) -> &'static str {
        "misc"
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
