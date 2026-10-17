//! sentimente (Blackbird/WMN): cat dating
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: david01, brahim01
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSentimente;

pub const URI: &str = "https://www.sentimente.com/amp/{account}.html";
pub const SAMPLE: Option<&str> = Some("david01");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Chat online with",
    m_code: 404,
    m_string: "HTTP Error code: 404. Resource not found",
};

#[async_trait]
impl l96a1_core::Platform for WmSentimente {
    fn key(&self) -> &'static str {
        "wm:sentimente"
    }
    fn name(&self) -> &'static str {
        "sentimente"
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
