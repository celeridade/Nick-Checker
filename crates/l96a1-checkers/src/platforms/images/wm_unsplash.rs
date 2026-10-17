//! unsplash (Blackbird/WMN): cat images
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: john, alex
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmUnsplash;

pub const URI: &str = "https://unsplash.com/@{account}";
pub const SAMPLE: Option<&str> = Some("john");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "| Unsplash Photo Community",
    m_code: 404,
    m_string: "Hm, the page you were looking for doesn't seem to exist anymore.",
};

#[async_trait]
impl l96a1_core::Platform for WmUnsplash {
    fn key(&self) -> &'static str {
        "wm:unsplash"
    }
    fn name(&self) -> &'static str {
        "unsplash"
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
