//! Watchmyfeed (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: jennifer-ann, shay-loveless
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmWatchmyfeed;

pub const URI: &str = "https://watchmyfeed.com/{account}";
pub const SAMPLE: Option<&str> = Some("jennifer-ann");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "SEND ME A TIP",
    m_code: 302,
    m_string: "",
};

#[async_trait]
impl l96a1_core::Platform for WmWatchmyfeed {
    fn key(&self) -> &'static str {
        "wm:watchmyfeed"
    }
    fn name(&self) -> &'static str {
        "Watchmyfeed"
    }
    fn group(&self) -> &'static str {
        "xx NSFW xx"
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
