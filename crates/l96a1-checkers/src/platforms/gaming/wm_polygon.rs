//! polygon (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: nicodeyo, Nicole_Clark
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPolygon;

pub const URI: &str = "https://www.polygon.com/users/{account}";
pub const SAMPLE: Option<&str> = Some("nicodeyo");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "- Polygon",
    m_code: 404,
    m_string: "404 Not found",
};

#[async_trait]
impl l96a1_core::Platform for WmPolygon {
    fn key(&self) -> &'static str {
        "wm:polygon"
    }
    fn name(&self) -> &'static str {
        "polygon"
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
