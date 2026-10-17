//! patch (Blackbird/WMN): cat news
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: dave, bob
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPatch;

pub const URI: &str = "https://patch.com/users/{account}";
pub const SAMPLE: Option<&str> = Some("dave");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<title>Patch User Profile",
    m_code: 404,
    m_string: "<title>Page not found</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmPatch {
    fn key(&self) -> &'static str {
        "wm:patch"
    }
    fn name(&self) -> &'static str {
        "patch"
    }
    fn group(&self) -> &'static str {
        "news"
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
