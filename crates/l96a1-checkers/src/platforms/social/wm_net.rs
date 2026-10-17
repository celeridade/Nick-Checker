//! Чатовка.net (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: ФорматА4, Alinka1313
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmNet;

pub const URI: &str = "https://chatovka.net/search?user_nick=+{account}&user_sex_m=on&user_sex_f=on";
pub const SAMPLE: Option<&str> = Some("ФорматА4");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "href=\"/user/",
    m_code: 200,
    m_string: "По Вашему запросу люди не найдены.",
};

#[async_trait]
impl l96a1_core::Platform for WmNet {
    fn key(&self) -> &'static str {
        "wm:чатовка.net"
    }
    fn name(&self) -> &'static str {
        "Чатовка.net"
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
