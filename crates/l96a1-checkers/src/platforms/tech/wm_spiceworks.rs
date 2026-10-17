//! SpiceWorks (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: spicerex, rod-it
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSpiceworks;

pub const URI: &str = "https://community.spiceworks.com/people/{account}";
pub const SAMPLE: Option<&str> = Some("spicerex");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Portfolio of IT Projects - Spiceworks",
    m_code: 404,
    m_string: "Page Not Found",
};

#[async_trait]
impl l96a1_core::Platform for WmSpiceworks {
    fn key(&self) -> &'static str {
        "wm:spiceworks"
    }
    fn name(&self) -> &'static str {
        "SpiceWorks"
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
