//! Greasy Fork (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: TScofield, gitscofield
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmGreasyFork;

pub const URI: &str = "https://greasyfork.org/en/users?q={account}";
pub const SAMPLE: Option<&str> = Some("TScofield");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"user-list\"",
    m_code: 200,
    m_string: "<p>No users!</p>",
};

#[async_trait]
impl l96a1_core::Platform for WmGreasyFork {
    fn key(&self) -> &'static str {
        "wm:greasy-fork"
    }
    fn name(&self) -> &'static str {
        "Greasy Fork"
    }
    fn group(&self) -> &'static str {
        "tech"
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
