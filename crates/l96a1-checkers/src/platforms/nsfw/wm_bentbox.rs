//! Bentbox (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: brockdoom, witchhouse, hotoptics
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmBentbox;

pub const URI: &str = "https://bentbox.co/{account}";
pub const SAMPLE: Option<&str> = Some("brockdoom");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<div id=\"user_bar\">",
    m_code: 200,
    m_string: "This user is currently not available",
};

#[async_trait]
impl l96a1_core::Platform for WmBentbox {
    fn key(&self) -> &'static str {
        "wm:bentbox"
    }
    fn name(&self) -> &'static str {
        "Bentbox"
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
