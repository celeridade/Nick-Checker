//! Poe.com (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: spdustin, PromptCase
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPoeCom;

pub const URI: &str = "https://poe.com/profile/{account}";
pub const SAMPLE: Option<&str> = Some("spdustin");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"__isNode\":\"PoeUser\"",
    m_code: 200,
    m_string: "\"user\":null",
};

#[async_trait]
impl l96a1_core::Platform for WmPoeCom {
    fn key(&self) -> &'static str {
        "wm:poe.com"
    }
    fn name(&self) -> &'static str {
        "Poe.com"
    }
    fn group(&self) -> &'static str {
        "tech"
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
