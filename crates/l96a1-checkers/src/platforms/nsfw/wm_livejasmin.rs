//! LiveJasmin (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: NaiaCollins, LoraClay
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmLivejasmin;

pub const URI: &str = "https://www.livejasmin.com/en/flash/get-performer-details/{account}";
pub const SAMPLE: Option<&str> = Some("NaiaCollins");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"success\":true",
    m_code: 200,
    m_string: "\"success\":false",
};

#[async_trait]
impl l96a1_core::Platform for WmLivejasmin {
    fn key(&self) -> &'static str {
        "wm:livejasmin"
    }
    fn name(&self) -> &'static str {
        "LiveJasmin"
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
