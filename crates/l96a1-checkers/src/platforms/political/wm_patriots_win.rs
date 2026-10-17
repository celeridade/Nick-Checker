//! Patriots Win (Blackbird/WMN): cat political
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: r3deleven, MemeFactory
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPatriotsWin;

pub const URI: &str = "https://patriots.win/api/v2/user/about.json?user={account}";
pub const SAMPLE: Option<&str> = Some("r3deleven");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"users\":[{",
    m_code: 200,
    m_string: "\"error\":\"invalid user\"",
};

#[async_trait]
impl l96a1_core::Platform for WmPatriotsWin {
    fn key(&self) -> &'static str {
        "wm:patriots-win"
    }
    fn name(&self) -> &'static str {
        "Patriots Win"
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
