//! Avid Community (Blackbird/WMN): cat music
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Thayne, Admin
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmAvidCommunity;

pub const URI: &str = "https://community.avid.com/members/{account}/default.aspx";
pub const SAMPLE: Option<&str> = Some("Thayne");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "My Activity",
    m_code: 302,
    m_string: "The user you requested cannot be found.",
};

#[async_trait]
impl l96a1_core::Platform for WmAvidCommunity {
    fn key(&self) -> &'static str {
        "wm:avid-community"
    }
    fn name(&self) -> &'static str {
        "Avid Community"
    }
    fn group(&self) -> &'static str {
        "music"
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
