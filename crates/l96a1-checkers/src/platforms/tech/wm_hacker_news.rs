//! Hacker News (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: mubix, hggh
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmHackerNews;

pub const URI: &str = "https://hacker-news.firebaseio.com/v0/user/{account}.json?print=pretty";
pub const SAMPLE: Option<&str> = Some("mubix");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"id\" :",
    m_code: 200,
    m_string: "null",
};

#[async_trait]
impl l96a1_core::Platform for WmHackerNews {
    fn key(&self) -> &'static str {
        "wm:hacker-news"
    }
    fn name(&self) -> &'static str {
        "Hacker News"
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
