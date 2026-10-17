//! Designspriation (Blackbird/WMN): cat art
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: sam, smith
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDesignspriation;

pub const URI: &str = "https://www.designspiration.com/{account}/";
pub const SAMPLE: Option<&str> = Some("sam");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "has discovered on Designspiration",
    m_code: 404,
    m_string: "Content Not Found",
};

#[async_trait]
impl l96a1_core::Platform for WmDesignspriation {
    fn key(&self) -> &'static str {
        "wm:designspriation"
    }
    fn name(&self) -> &'static str {
        "Designspriation"
    }
    fn group(&self) -> &'static str {
        "art"
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
