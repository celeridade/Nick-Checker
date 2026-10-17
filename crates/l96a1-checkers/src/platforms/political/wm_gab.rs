//! Gab (Blackbird/WMN): cat political
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: RealMarjorieGreene, LaurenBoebert
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmGab;

pub const URI: &str = "https://gab.com/api/v1/account_by_username/{account}";
pub const SAMPLE: Option<&str> = Some("RealMarjorieGreene");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"id\":",
    m_code: 404,
    m_string: "\"error\":\"Record not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmGab {
    fn key(&self) -> &'static str {
        "wm:gab"
    }
    fn name(&self) -> &'static str {
        "Gab"
    }
    fn group(&self) -> &'static str {
        "political"
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
