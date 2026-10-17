//! Pinterest (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: test123, frickcollection
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPinterest;

pub const URI: &str = "https://www.pinterest.com/{account}/";
pub const SAMPLE: Option<&str> = Some("test123");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: " - Profile | Pinterest",
    m_code: 301,
    m_string: "id=\"home-main-title",
};

#[async_trait]
impl l96a1_core::Platform for WmPinterest {
    fn key(&self) -> &'static str {
        "wm:pinterest"
    }
    fn name(&self) -> &'static str {
        "Pinterest"
    }
    fn group(&self) -> &'static str {
        "social"
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
