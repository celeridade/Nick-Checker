//! RumbleUser (Blackbird/WMN): cat political
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: SimonParkes, djmrmusic
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmRumbleuser;

pub const URI: &str = "https://rumble.com/user/{account}";
pub const SAMPLE: Option<&str> = Some("SimonParkes");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "href=https://rumble.com/user/",
    m_code: 404,
    m_string: "404 error, this page does not exist",
};

#[async_trait]
impl l96a1_core::Platform for WmRumbleuser {
    fn key(&self) -> &'static str {
        "wm:rumbleuser"
    }
    fn name(&self) -> &'static str {
        "RumbleUser"
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
