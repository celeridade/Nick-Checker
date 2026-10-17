//! Peerlist (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: taher_max_, naymurr
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPeerlist;

pub const URI: &str = "https://peerlist.io/api/v1/users/profile/tldr?handle={account}";
pub const SAMPLE: Option<&str> = Some("taher_max_");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"tldr\":",
    m_code: 500,
    m_string: "\"message\":\"User Not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmPeerlist {
    fn key(&self) -> &'static str {
        "wm:peerlist"
    }
    fn name(&self) -> &'static str {
        "Peerlist"
    }
    fn group(&self) -> &'static str {
        "coding"
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
