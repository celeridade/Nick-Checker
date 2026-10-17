//! Subscribestar (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: missmoonified, honey
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSubscribestar;

pub const URI: &str = "https://subscribestar.adult/{account}";
pub const SAMPLE: Option<&str> = Some("missmoonified");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "CREATOR STATS",
    m_code: 404,
    m_string: "WE ARE SORRY, THE PAGE YOU REQUESTED CANNOT BE FOUND",
};

#[async_trait]
impl l96a1_core::Platform for WmSubscribestar {
    fn key(&self) -> &'static str {
        "wm:subscribestar"
    }
    fn name(&self) -> &'static str {
        "Subscribestar"
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
