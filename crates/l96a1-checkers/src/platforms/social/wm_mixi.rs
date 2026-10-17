//! Mixi (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: 2854333, 19123
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMixi;

pub const URI: &str = "https://mixi.jp/view_community.pl?id={account}";
pub const SAMPLE: Option<&str> = Some("2854333");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "| mixiコミュニティ</title>",
    m_code: 200,
    m_string: "データがありません",
};

#[async_trait]
impl l96a1_core::Platform for WmMixi {
    fn key(&self) -> &'static str {
        "wm:mixi"
    }
    fn name(&self) -> &'static str {
        "Mixi"
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
