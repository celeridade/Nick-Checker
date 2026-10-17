//! Habtium (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: diegjeremy, suigetsu
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmHabtium;

pub const URI: &str = "https://habtium.es/{account}";
pub const SAMPLE: Option<&str> = Some("diegjeremy");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<div class=\"profile-info",
    m_code: 404,
    m_string: "<h1 class=\"section red\">Oops!",
};

#[async_trait]
impl l96a1_core::Platform for WmHabtium {
    fn key(&self) -> &'static str {
        "wm:habtium"
    }
    fn name(&self) -> &'static str {
        "Habtium"
    }
    fn group(&self) -> &'static str {
        "gaming"
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
