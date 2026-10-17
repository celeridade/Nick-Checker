//! meet me (Blackbird/WMN): cat dating
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: john, marsha
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMeetMe;

pub const URI: &str = "https://www.meetme.com/{account}";
pub const SAMPLE: Option<&str> = Some("john");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<title>Meet people like ",
    m_code: 302,
    m_string: "<title>MeetMe - Chat and Meet New People</title",
};

#[async_trait]
impl l96a1_core::Platform for WmMeetMe {
    fn key(&self) -> &'static str {
        "wm:meet-me"
    }
    fn name(&self) -> &'static str {
        "meet me"
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
