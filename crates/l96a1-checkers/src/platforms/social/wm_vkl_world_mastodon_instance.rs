//! Vkl.world (Mastodon Instance) (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: king, aniver
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmVklWorldMastodonInstance;

pub const URI: &str = "https://vkl.world/api/v1/accounts/lookup?acct={account}";
pub const SAMPLE: Option<&str> = Some("king");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "display_name",
    m_code: 404,
    m_string: "Record not found",
};

#[async_trait]
impl l96a1_core::Platform for WmVklWorldMastodonInstance {
    fn key(&self) -> &'static str {
        "wm:vkl.world-(mastodon-instance)"
    }
    fn name(&self) -> &'static str {
        "Vkl.world (Mastodon Instance)"
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
