//! MapMyTracks (Blackbird/WMN): cat health
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: ulirad, CBSloan
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMapmytracks;

pub const URI: &str = "https://www.mapmytracks.com/{account}";
pub const SAMPLE: Option<&str> = Some("ulirad");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Daily distance this week",
    m_code: 302,
    m_string: "Outside together",
};

#[async_trait]
impl l96a1_core::Platform for WmMapmytracks {
    fn key(&self) -> &'static str {
        "wm:mapmytracks"
    }
    fn name(&self) -> &'static str {
        "MapMyTracks"
    }
    fn group(&self) -> &'static str {
        "health"
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
