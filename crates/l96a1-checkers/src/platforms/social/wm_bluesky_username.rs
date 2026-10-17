//! Bluesky Username (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: john, mark
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmBlueskyUsername;

pub const URI: &str = "https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor={account}.bsky.social";
pub const SAMPLE: Option<&str> = Some("john");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"handle\":\"",
    m_code: 400,
    m_string: "\"message\":\"Profile not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmBlueskyUsername {
    fn key(&self) -> &'static str {
        "wm:bluesky-username"
    }
    fn name(&self) -> &'static str {
        "Bluesky Username"
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
