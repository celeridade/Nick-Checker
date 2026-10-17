//! carrd.co (Blackbird/WMN): cat business
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: liam, peter
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmCarrdCo;

pub const URI: &str = "https://{account}.carrd.co";
pub const SAMPLE: Option<&str> = Some("liam");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "( Made with Carrd )",
    m_code: 404,
    m_string: "Sorry, the requested page could not be found.",
};

#[async_trait]
impl l96a1_core::Platform for WmCarrdCo {
    fn key(&self) -> &'static str {
        "wm:carrd.co"
    }
    fn name(&self) -> &'static str {
        "carrd.co"
    }
    fn group(&self) -> &'static str {
        "business"
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
