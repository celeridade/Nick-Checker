//! Voice123 (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: dottovuu, maheshsaha1992
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmVoice123;

pub const URI: &str = "https://voice123.com/api/providers/search/{account}";
pub const SAMPLE: Option<&str> = Some("dottovuu");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "user_id",
    m_code: 200,
    m_string: "[]",
};

#[async_trait]
impl l96a1_core::Platform for WmVoice123 {
    fn key(&self) -> &'static str {
        "wm:voice123"
    }
    fn name(&self) -> &'static str {
        "Voice123"
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
