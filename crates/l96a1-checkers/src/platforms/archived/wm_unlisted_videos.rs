//! Unlisted Videos (Blackbird/WMN): cat archived
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: emudshit, hirumaredx
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmUnlistedVideos;

pub const URI: &str = "https://unlistedvideos.com/search.php?user={account}";
pub const SAMPLE: Option<&str> = Some("emudshit");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Date submitted",
    m_code: 200,
    m_string: "content=\"\"/>",
};

#[async_trait]
impl l96a1_core::Platform for WmUnlistedVideos {
    fn key(&self) -> &'static str {
        "wm:unlisted-videos"
    }
    fn name(&self) -> &'static str {
        "Unlisted Videos"
    }
    fn group(&self) -> &'static str {
        "archived"
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
