//! Dota2.ru (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: narancha, Darkness whisper
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDota2Ru;

pub const URI: &str = "https://dota2.ru/forum/search/?type=user&keywords={account}&sort_by=username";
pub const SAMPLE: Option<&str> = Some("narancha");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"forum-section__item forum-section__item--first\"",
    m_code: 200,
    m_string: "id=\"no-activity-posts\"",
};

#[async_trait]
impl l96a1_core::Platform for WmDota2Ru {
    fn key(&self) -> &'static str {
        "wm:dota2.ru"
    }
    fn name(&self) -> &'static str {
        "Dota2.ru"
    }
    fn group(&self) -> &'static str {
        "gaming"
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
