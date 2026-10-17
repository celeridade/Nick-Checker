//! Venmo (Blackbird/WMN): cat finance
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: John-Goolsby-8, kate-mura
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmVenmo;

pub const URI: &str = "https://account.venmo.com/u/{account}";
pub const SAMPLE: Option<&str> = Some("John-Goolsby-8");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "profileInfo_username__",
    m_code: 404,
    m_string: "Sorry, the page you requested does not exist!",
};

#[async_trait]
impl l96a1_core::Platform for WmVenmo {
    fn key(&self) -> &'static str {
        "wm:venmo"
    }
    fn name(&self) -> &'static str {
        "Venmo"
    }
    fn group(&self) -> &'static str {
        "finance"
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
