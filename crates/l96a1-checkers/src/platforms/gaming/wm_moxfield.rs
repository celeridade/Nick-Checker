//! Moxfield (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: gamer, Carlos01
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMoxfield;

pub const URI: &str = "https://api2.moxfield.com/v1/users/{account}";
pub const SAMPLE: Option<&str> = Some("gamer");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"userName\":",
    m_code: 404,
    m_string: "\"status\":404",
};

#[async_trait]
impl l96a1_core::Platform for WmMoxfield {
    fn key(&self) -> &'static str {
        "wm:moxfield"
    }
    fn name(&self) -> &'static str {
        "Moxfield"
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
