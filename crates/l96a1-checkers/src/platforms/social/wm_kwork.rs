//! Kwork (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: ilkarkarakurt, sergeymeshiy
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmKwork;

pub const URI: &str = "https://kwork.ru/user_kworks/{account}";
pub const SAMPLE: Option<&str> = Some("ilkarkarakurt");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"success\":true",
    m_code: 200,
    m_string: "\"success\":false",
};

#[async_trait]
impl l96a1_core::Platform for WmKwork {
    fn key(&self) -> &'static str {
        "wm:kwork"
    }
    fn name(&self) -> &'static str {
        "Kwork"
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
