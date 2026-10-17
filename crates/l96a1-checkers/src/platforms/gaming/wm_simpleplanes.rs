//! SimplePlanes (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: realSavageMan, Jundroo, john
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSimpleplanes;

pub const URI: &str = "https://www.simpleplanes.com/u/{account}";
pub const SAMPLE: Option<&str> = Some("realSavageMan");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<h5>joined",
    m_code: 302,
    m_string: "<title>SimplePlanes Airplanes</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmSimpleplanes {
    fn key(&self) -> &'static str {
        "wm:simpleplanes"
    }
    fn name(&self) -> &'static str {
        "SimplePlanes"
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
