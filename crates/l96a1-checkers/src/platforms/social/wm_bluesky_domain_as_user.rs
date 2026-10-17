//! Bluesky Domain as User (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: bsky.app, webbreacher.com
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmBlueskyDomainAsUser;

pub const URI: &str = "https://bsky.app/profile/{account}";
pub const SAMPLE: Option<&str> = Some("bsky.app");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "on Bluesky</title>",
    m_code: 200,
    m_string: "<title>Bluesky</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmBlueskyDomainAsUser {
    fn key(&self) -> &'static str {
        "wm:bluesky-domain-as-user"
    }
    fn name(&self) -> &'static str {
        "Bluesky Domain as User"
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
