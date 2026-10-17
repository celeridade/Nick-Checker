//! Fodors Forum (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: jdstraveler, gooster
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFodorsForum;

pub const URI: &str = "https://www.fodors.com/community/profile/{account}/forum-activity";
pub const SAMPLE: Option<&str> = Some("jdstraveler");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "User Profile | Fodor’s Travel</title>",
    m_code: 302,
    m_string: "Plan Your Trip Online</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmFodorsForum {
    fn key(&self) -> &'static str {
        "wm:fodors-forum"
    }
    fn name(&self) -> &'static str {
        "Fodors Forum"
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
