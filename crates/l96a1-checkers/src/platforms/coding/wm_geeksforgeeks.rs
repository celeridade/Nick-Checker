//! GeeksForGeeks (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: nath_789, harshrajsinghsiwan, igovindindia
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmGeeksforgeeks;

pub const URI: &str = "https://authapi.geeksforgeeks.org/api-get/user-profile-info/?handle={account}";
pub const SAMPLE: Option<&str> = Some("nath_789");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"message\":\"data retrieved successfully\"",
    m_code: 400,
    m_string: "\"message\":\"User not found!\"",
};

#[async_trait]
impl l96a1_core::Platform for WmGeeksforgeeks {
    fn key(&self) -> &'static str {
        "wm:geeksforgeeks"
    }
    fn name(&self) -> &'static str {
        "GeeksForGeeks"
    }
    fn group(&self) -> &'static str {
        "coding"
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
