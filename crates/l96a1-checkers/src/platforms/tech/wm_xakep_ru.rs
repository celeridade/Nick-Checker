//! Xakep.ru (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: tr3harder, stariy
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmXakepRu;

pub const URI: &str = "https://xakep.ru/author/{account}/";
pub const SAMPLE: Option<&str> = Some("tr3harder");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "authorBlock-avatar",
    m_code: 404,
    m_string: "Страница не найдена",
};

#[async_trait]
impl l96a1_core::Platform for WmXakepRu {
    fn key(&self) -> &'static str {
        "wm:xakep.ru"
    }
    fn name(&self) -> &'static str {
        "Xakep.ru"
    }
    fn group(&self) -> &'static str {
        "tech"
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
