//! FACEIT (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: s1mple, w0nderful
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFaceit;

pub const URI: &str = "https://www.faceit.com/api/users/v1/nicknames/{account}";
pub const SAMPLE: Option<&str> = Some("s1mple");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"result\":\"OK\"",
    m_code: 404,
    m_string: "\"message\":\"user not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmFaceit {
    fn key(&self) -> &'static str {
        "wm:faceit"
    }
    fn name(&self) -> &'static str {
        "FACEIT"
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
