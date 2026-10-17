//! Periscope (Blackbird/WMN): cat video
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: john, test
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPeriscope;

pub const URI: &str = "https://www.periscope.tv/{account}";
pub const SAMPLE: Option<&str> = Some("john");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<label>Followers",
    m_code: 404,
    m_string: "Sorry, this page doesn’t exist",
};

#[async_trait]
impl l96a1_core::Platform for WmPeriscope {
    fn key(&self) -> &'static str {
        "wm:periscope"
    }
    fn name(&self) -> &'static str {
        "Periscope"
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
