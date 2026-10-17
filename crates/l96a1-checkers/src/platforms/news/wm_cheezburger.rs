//! CHEEZburger (Blackbird/WMN): cat news
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: BarMorHazut, inessoubrie
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmCheezburger;

pub const URI: &str = "https://cheezburger.com/Editor/{account}";
pub const SAMPLE: Option<&str> = Some("BarMorHazut");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<title>Recent Posts from",
    m_code: 404,
    m_string: "<title>404 Not Found</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmCheezburger {
    fn key(&self) -> &'static str {
        "wm:cheezburger"
    }
    fn name(&self) -> &'static str {
        "CHEEZburger"
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
