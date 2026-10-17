//! Fur Affinity (Blackbird/WMN): cat images
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: karintina, mikrogoat
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFurAffinity;

pub const URI: &str = "https://www.furaffinity.net/user/{account}/";
pub const SAMPLE: Option<&str> = Some("karintina");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<userpage-nav-header>",
    m_code: 200,
    m_string: "<h2>System Error</h2>",
};

#[async_trait]
impl l96a1_core::Platform for WmFurAffinity {
    fn key(&self) -> &'static str {
        "wm:fur-affinity"
    }
    fn name(&self) -> &'static str {
        "Fur Affinity"
    }
    fn group(&self) -> &'static str {
        "images"
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
