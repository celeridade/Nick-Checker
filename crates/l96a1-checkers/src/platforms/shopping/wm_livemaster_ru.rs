//! livemaster.ru (Blackbird/WMN): cat shopping
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: redart, ellentoy
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmLivemasterRu;

pub const URI: &str = "https://www.livemaster.ru/{account}";
pub const SAMPLE: Option<&str> = Some("redart");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<title>Магазин мастера",
    m_code: 404,
    m_string: "<title>Вы попали на несуществующую страницу",
};

#[async_trait]
impl l96a1_core::Platform for WmLivemasterRu {
    fn key(&self) -> &'static str {
        "wm:livemaster.ru"
    }
    fn name(&self) -> &'static str {
        "livemaster.ru"
    }
    fn group(&self) -> &'static str {
        "shopping"
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
