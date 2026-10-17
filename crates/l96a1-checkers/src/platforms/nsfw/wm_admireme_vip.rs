//! AdmireMe.VIP (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: justjessicarabbit, savannah250xo
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmAdmiremeVip;

pub const URI: &str = "https://admireme.vip/{account}/";
pub const SAMPLE: Option<&str> = Some("justjessicarabbit");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "creator-stat subscriber",
    m_code: 404,
    m_string: "<title>Page Not Found |",
};

#[async_trait]
impl l96a1_core::Platform for WmAdmiremeVip {
    fn key(&self) -> &'static str {
        "wm:admireme.vip"
    }
    fn name(&self) -> &'static str {
        "AdmireMe.VIP"
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
