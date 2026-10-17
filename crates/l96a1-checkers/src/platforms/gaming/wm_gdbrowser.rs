//! GDBrowser (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: SorkoPiko, Subwoofer
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmGdbrowser;

pub const URI: &str = "https://gdbrowser.com/api/profile/{account}";
pub const SAMPLE: Option<&str> = Some("SorkoPiko");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"accountID\":",
    m_code: 500,
    m_string: "-1",
};

#[async_trait]
impl l96a1_core::Platform for WmGdbrowser {
    fn key(&self) -> &'static str {
        "wm:gdbrowser"
    }
    fn name(&self) -> &'static str {
        "GDBrowser"
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
