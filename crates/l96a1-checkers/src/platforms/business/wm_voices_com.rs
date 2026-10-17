//! Voices.com (Blackbird/WMN): cat business
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: briankirchoff, bryankopta
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmVoicesCom;

pub const URI: &str = "https://www.voices.com/profile/{account}/";
pub const SAMPLE: Option<&str> = Some("briankirchoff");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Last Online</h3>",
    m_code: 301,
    m_string: "Try going back to the previous page or see below for more options",
};

#[async_trait]
impl l96a1_core::Platform for WmVoicesCom {
    fn key(&self) -> &'static str {
        "wm:voices.com"
    }
    fn name(&self) -> &'static str {
        "Voices.com"
    }
    fn group(&self) -> &'static str {
        "business"
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
