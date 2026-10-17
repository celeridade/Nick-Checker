//! Magnific (Blackbird/WMN): cat art
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: membiz, LaraRed
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMagnific;

pub const URI: &str = "https://www.magnific.com/app/api/community/creator-info-by-name?creator_name={account}&lang=en_US";
pub const SAMPLE: Option<&str> = Some("membiz");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"creator\":",
    m_code: 404,
    m_string: "\"message\":\"Creator not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmMagnific {
    fn key(&self) -> &'static str {
        "wm:magnific"
    }
    fn name(&self) -> &'static str {
        "Magnific"
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
