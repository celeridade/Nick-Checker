//! Oper.ru (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Goblin, Баир Иринчеев
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmOperRu;

pub const URI: &str = "https://oper.ru/visitors/info.php?t={account}";
pub const SAMPLE: Option<&str> = Some("Goblin");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Информация о пользователе",
    m_code: 200,
    m_string: "Нет такого пользователя",
};

#[async_trait]
impl l96a1_core::Platform for WmOperRu {
    fn key(&self) -> &'static str {
        "wm:oper.ru"
    }
    fn name(&self) -> &'static str {
        "Oper.ru"
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
