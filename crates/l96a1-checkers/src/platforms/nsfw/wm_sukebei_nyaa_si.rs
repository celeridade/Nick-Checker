//! sukebei.nyaa.si (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: kouhy76, Rektr0
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSukebeiNyaaSi;

pub const URI: &str = "https://sukebei.nyaa.si/user/{account}";
pub const SAMPLE: Option<&str> = Some("kouhy76");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "'s torrents",
    m_code: 404,
    m_string: "404 Not Found",
};

#[async_trait]
impl l96a1_core::Platform for WmSukebeiNyaaSi {
    fn key(&self) -> &'static str {
        "wm:sukebei.nyaa.si"
    }
    fn name(&self) -> &'static str {
        "sukebei.nyaa.si"
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
