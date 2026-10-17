//! palnet (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: trincowski-pt, anggreklestari
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPalnet;

pub const URI: &str = "https://www.palnet.io/@{account}/";
pub const SAMPLE: Option<&str> = Some("trincowski-pt");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"profile-cover\"",
    m_code: 404,
    m_string: "Unknown user account!",
};

#[async_trait]
impl l96a1_core::Platform for WmPalnet {
    fn key(&self) -> &'static str {
        "wm:palnet"
    }
    fn name(&self) -> &'static str {
        "palnet"
    }
    fn group(&self) -> &'static str {
        "social"
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
