//! Thetattooforum (Blackbird/WMN): cat art
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: mixdop, modifyielts
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmThetattooforum;

pub const URI: &str = "https://www.thetattooforum.com/members/{account}/";
pub const SAMPLE: Option<&str> = Some("mixdop");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Insert This Gallery",
    m_code: 500,
    m_string: "We’re sorry",
};

#[async_trait]
impl l96a1_core::Platform for WmThetattooforum {
    fn key(&self) -> &'static str {
        "wm:thetattooforum"
    }
    fn name(&self) -> &'static str {
        "Thetattooforum"
    }
    fn group(&self) -> &'static str {
        "art"
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
