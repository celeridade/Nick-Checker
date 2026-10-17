//! LevelBlue (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: BotnetExposer, jamesbrine
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmLevelblue;

pub const URI: &str = "https://otx.alienvault.com/otxapi/auth/validate?username={account}";
pub const SAMPLE: Option<&str> = Some("BotnetExposer");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 400,
    e_string: "\"username\": [\"This username is already taken\"]",
    m_code: 200,
    m_string: "{}",
};

#[async_trait]
impl l96a1_core::Platform for WmLevelblue {
    fn key(&self) -> &'static str {
        "wm:levelblue"
    }
    fn name(&self) -> &'static str {
        "LevelBlue"
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
