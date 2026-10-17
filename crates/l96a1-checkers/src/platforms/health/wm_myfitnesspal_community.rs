//! MyFitnessPal Community (Blackbird/WMN): cat health
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: malibu927, L1zardQueen
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMyfitnesspalCommunity;

pub const URI: &str = "https://community.myfitnesspal.com/en/profile/{account}";
pub const SAMPLE: Option<&str> = Some("malibu927");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: ">Last Active<",
    m_code: 404,
    m_string: "User Not Found",
};

#[async_trait]
impl l96a1_core::Platform for WmMyfitnesspalCommunity {
    fn key(&self) -> &'static str {
        "wm:myfitnesspal-community"
    }
    fn name(&self) -> &'static str {
        "MyFitnessPal Community"
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
