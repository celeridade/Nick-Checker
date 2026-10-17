//! skeb (Blackbird/WMN): cat art
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: eipuru_, sime064
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSkeb;

pub const URI: &str = "https://skeb.jp/@{account}";
pub const SAMPLE: Option<&str> = Some("eipuru_");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: ") | Skeb",
    m_code: 503,
    m_string: "Skeb - Request Box",
};

#[async_trait]
impl l96a1_core::Platform for WmSkeb {
    fn key(&self) -> &'static str {
        "wm:skeb"
    }
    fn name(&self) -> &'static str {
        "skeb"
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
