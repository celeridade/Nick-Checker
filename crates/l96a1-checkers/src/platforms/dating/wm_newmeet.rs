//! newmeet (Blackbird/WMN): cat dating
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Harmonie06, Bach007
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmNewmeet;

pub const URI: &str = "https://www.newmeet.com/en/profile/{account}/";
pub const SAMPLE: Option<&str> = Some("Harmonie06");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<h2>The profile of",
    m_code: 200,
    m_string: "Chat with , , ,  - ",
};

#[async_trait]
impl l96a1_core::Platform for WmNewmeet {
    fn key(&self) -> &'static str {
        "wm:newmeet"
    }
    fn name(&self) -> &'static str {
        "newmeet"
    }
    fn group(&self) -> &'static str {
        "dating"
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
