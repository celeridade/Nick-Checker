//! XNXX (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: john, mumrra
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmXnxx;

pub const URI: &str = "https://www.xnxx.com/mobile/profile/{account}";
pub const SAMPLE: Option<&str> = Some("john");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<table id=\"profile\">",
    m_code: 400,
    m_string: "<title>Bad request",
};

#[async_trait]
impl l96a1_core::Platform for WmXnxx {
    fn key(&self) -> &'static str {
        "wm:xnxx"
    }
    fn name(&self) -> &'static str {
        "XNXX"
    }
    fn group(&self) -> &'static str {
        "xx NSFW xx"
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
