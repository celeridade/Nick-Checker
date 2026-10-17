//! 247sports (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Travis Branham, BlairAngulo
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct Wm247sports;

pub const URI: &str = "https://247sports.com/User/{account}/";
pub const SAMPLE: Option<&str> = Some("Travis Branham");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "staff-header-container",
    m_code: 404,
    m_string: "<title>247Sports</title>",
};

#[async_trait]
impl l96a1_core::Platform for Wm247sports {
    fn key(&self) -> &'static str {
        "wm:247sports"
    }
    fn name(&self) -> &'static str {
        "247sports"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
