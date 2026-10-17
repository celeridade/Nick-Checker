//! YouTube User2 (Blackbird/WMN): cat video
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: tactical-systems, CybersecurityMeg
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmYoutubeUser2;

pub const URI: &str = "https://www.youtube.com/@{account}";
pub const SAMPLE: Option<&str> = Some("tactical-systems");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "canonicalBaseUrl",
    m_code: 404,
    m_string: "<title>404 Not Found</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmYoutubeUser2 {
    fn key(&self) -> &'static str {
        "wm:youtube-user2"
    }
    fn name(&self) -> &'static str {
        "YouTube User2"
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
