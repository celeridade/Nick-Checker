//! Sepia Search (PeerTube) (Blackbird/WMN): cat video
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: sustainable_sailing, t_mattei
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSepiaSearchPeertube;

pub const URI: &str = "https://sepiasearch.org/api/v1/search/video-channels?search=%22{account}%22&start=0&count=10";
pub const SAMPLE: Option<&str> = Some("sustainable_sailing");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"data\":[{",
    m_code: 200,
    m_string: "\"data\":[]",
};

#[async_trait]
impl l96a1_core::Platform for WmSepiaSearchPeertube {
    fn key(&self) -> &'static str {
        "wm:sepia-search-(peertube)"
    }
    fn name(&self) -> &'static str {
        "Sepia Search (PeerTube)"
    }
    fn group(&self) -> &'static str {
        "video"
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
