//! Fabswingers (Blackbird/WMN): cat dating
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: justusboth2013, hellfireclub, fabswingers.com
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFabswingers;

pub const URI: &str = "https://www.fabswingers.com/profile/{account}";
pub const SAMPLE: Option<&str> = Some("justusboth2013");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "View Profile",
    m_code: 200,
    m_string: "The user you tried to view doesn't seem to be on the site any more",
};

#[async_trait]
impl l96a1_core::Platform for WmFabswingers {
    fn key(&self) -> &'static str {
        "wm:fabswingers"
    }
    fn name(&self) -> &'static str {
        "Fabswingers"
    }
    fn group(&self) -> &'static str {
        "dating"
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
