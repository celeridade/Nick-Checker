//! NaturalNews (Blackbird/WMN): cat political
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: jdheyes, healthranger
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmNaturalnews;

pub const URI: &str = "https://naturalnews.com/author/{account}/";
pub const SAMPLE: Option<&str> = Some("jdheyes");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "All posts by",
    m_code: 200,
    m_string: "The page you are looking for cannot be found or is no longer available.",
};

#[async_trait]
impl l96a1_core::Platform for WmNaturalnews {
    fn key(&self) -> &'static str {
        "wm:naturalnews"
    }
    fn name(&self) -> &'static str {
        "NaturalNews"
    }
    fn group(&self) -> &'static str {
        "political"
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
