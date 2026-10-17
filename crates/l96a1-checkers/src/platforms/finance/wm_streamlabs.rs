//! StreamLabs (Blackbird/WMN): cat finance
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: veibae, cutie_cori
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmStreamlabs;

pub const URI: &str = "https://streamlabs.com/api/v6/user/{account}";
pub const SAMPLE: Option<&str> = Some("veibae");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"id\":",
    m_code: 401,
    m_string: "<title>Unauthorized</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmStreamlabs {
    fn key(&self) -> &'static str {
        "wm:streamlabs"
    }
    fn name(&self) -> &'static str {
        "StreamLabs"
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
