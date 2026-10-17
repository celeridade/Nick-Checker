//! Nih Buat Jajan (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: banyusadewa, danirachmat
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmNihBuatJajan;

pub const URI: &str = "https://www.nihbuatjajan.com/manifest/{account}.json";
pub const SAMPLE: Option<&str> = Some("banyusadewa");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"name\":",
    m_code: 404,
    m_string: "\"error\":\"Creator not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmNihBuatJajan {
    fn key(&self) -> &'static str {
        "wm:nih-buat-jajan"
    }
    fn name(&self) -> &'static str {
        "Nih Buat Jajan"
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
