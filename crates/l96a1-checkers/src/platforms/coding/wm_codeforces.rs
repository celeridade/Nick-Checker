//! Codeforces (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Abdul01, Abdullah
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmCodeforces;

pub const URI: &str = "https://codeforces.com/api/user.info?handles={account}";
pub const SAMPLE: Option<&str> = Some("Abdul01");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"status\":\"OK\"",
    m_code: 400,
    m_string: "\"status\":\"FAILED\"",
};

#[async_trait]
impl l96a1_core::Platform for WmCodeforces {
    fn key(&self) -> &'static str {
        "wm:codeforces"
    }
    fn name(&self) -> &'static str {
        "Codeforces"
    }
    fn group(&self) -> &'static str {
        "coding"
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
