//! icq-chat (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: brookenora.54, bigdaddy.77
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmIcqChat;

pub const URI: &str = "https://icq.icqchat.co/members/{account}/";
pub const SAMPLE: Option<&str> = Some("brookenora.54");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"memberHeader-mainContent",
    m_code: 404,
    m_string: "<title>Oops! We ran into some problems.",
};

#[async_trait]
impl l96a1_core::Platform for WmIcqChat {
    fn key(&self) -> &'static str {
        "wm:icq-chat"
    }
    fn name(&self) -> &'static str {
        "icq-chat"
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
