//! warriorforum (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: alex, discrat
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmWarriorforum;

pub const URI: &str = "https://www.warriorforum.com/members/{account}.html";
pub const SAMPLE: Option<&str> = Some("alex");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Last Activity:",
    m_code: 400,
    m_string: "Oops | Warrior Forum -",
};

#[async_trait]
impl l96a1_core::Platform for WmWarriorforum {
    fn key(&self) -> &'static str {
        "wm:warriorforum"
    }
    fn name(&self) -> &'static str {
        "warriorforum"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
