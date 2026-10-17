//! Flickr (Blackbird/WMN): cat images
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: glaciernps, test
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFlickr;

pub const URI: &str = "https://www.flickr.com/photos/{account}/";
pub const SAMPLE: Option<&str> = Some("glaciernps");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "| Flickr",
    m_code: 404,
    m_string: "",
};

#[async_trait]
impl l96a1_core::Platform for WmFlickr {
    fn key(&self) -> &'static str {
        "wm:flickr"
    }
    fn name(&self) -> &'static str {
        "Flickr"
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
