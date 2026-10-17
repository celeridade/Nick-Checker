//! Playstation Network (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: SlimShaggy18, ikemenzi
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPlaystationNetwork;

pub const URI: &str = "https://psnprofiles.com/xhr/search/users?q={account}";
pub const SAMPLE: Option<&str> = Some("SlimShaggy18");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<div class=\"progress-bar small level\">",
    m_code: 200,
    m_string: "We couldn't find anything ",
};

#[async_trait]
impl l96a1_core::Platform for WmPlaystationNetwork {
    fn key(&self) -> &'static str {
        "wm:playstation-network"
    }
    fn name(&self) -> &'static str {
        "Playstation Network"
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
