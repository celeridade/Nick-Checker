//! TikTok (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: gordonramsayofficial, pookiebear73
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmTiktok;

pub const URI: &str = "https://www.tiktok.com/oembed?url=https://www.tiktok.com/@{account}";
pub const SAMPLE: Option<&str> = Some("gordonramsayofficial");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"author_url\":",
    m_code: 400,
    m_string: "\"code\":400",
};

#[async_trait]
impl l96a1_core::Platform for WmTiktok {
    fn key(&self) -> &'static str {
        "wm:tiktok"
    }
    fn name(&self) -> &'static str {
        "TikTok"
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
