//! Home Design 3D (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: carlos01, paul
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmHomeDesign3d;

pub const URI: &str = "https://www.homedesign3d.net/user/{account}";
pub const SAMPLE: Option<&str> = Some("carlos01");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "id=\"user_content\"",
    m_code: 302,
    m_string: "<title>Community</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmHomeDesign3d {
    fn key(&self) -> &'static str {
        "wm:home-design-3d"
    }
    fn name(&self) -> &'static str {
        "Home Design 3D"
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
