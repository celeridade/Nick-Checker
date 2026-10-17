//! Massage Anywhere (Blackbird/WMN): cat health
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: lorilmccluskey, LomiNYC
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMassageAnywhere;

pub const URI: &str = "https://www.massageanywhere.com/profile/{account}";
pub const SAMPLE: Option<&str> = Some("lorilmccluskey");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<title>MassageAnywhere.com Profile for ",
    m_code: 200,
    m_string: "<title>MassageAnywhere.com: Search Results</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmMassageAnywhere {
    fn key(&self) -> &'static str {
        "wm:massage-anywhere"
    }
    fn name(&self) -> &'static str {
        "Massage Anywhere"
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
