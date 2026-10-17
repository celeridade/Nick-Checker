//! Roblox (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: LeetPawn, elephant459
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmRoblox;

pub const URI: &str = "https://auth.roblox.com/v1/usernames/validate?username={account}&birthday=2019-12-31T23:00:00.000Z";
pub const SAMPLE: Option<&str> = Some("LeetPawn");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Username is already in use",
    m_code: 200,
    m_string: "Username is valid",
};

#[async_trait]
impl l96a1_core::Platform for WmRoblox {
    fn key(&self) -> &'static str {
        "wm:roblox"
    }
    fn name(&self) -> &'static str {
        "Roblox"
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
