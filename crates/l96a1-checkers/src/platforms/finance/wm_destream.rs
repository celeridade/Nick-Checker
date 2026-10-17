//! destream (Blackbird/WMN): cat finance
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: skromnuy_fifa, wudjer
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDestream;

pub const URI: &str = "https://api.destream.net/siteapi/v2/live/details/{account}";
pub const SAMPLE: Option<&str> = Some("skromnuy_fifa");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"userName\":",
    m_code: 400,
    m_string: "\"errorMessage\":\"Error happened.\"",
};

#[async_trait]
impl l96a1_core::Platform for WmDestream {
    fn key(&self) -> &'static str {
        "wm:destream"
    }
    fn name(&self) -> &'static str {
        "destream"
    }
    fn group(&self) -> &'static str {
        "finance"
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
