//! igromania (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: bob, blue
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmIgromania;

pub const URI: &str = "https://forum.igromania.ru/member.php?username={account}";
pub const SAMPLE: Option<&str> = Some("bob");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Форум Игромании - Просмотр профиля:",
    m_code: 200,
    m_string: "Пользователь не зарегистрирован и не имеет профиля для просмотра.",
};

#[async_trait]
impl l96a1_core::Platform for WmIgromania {
    fn key(&self) -> &'static str {
        "wm:igromania"
    }
    fn name(&self) -> &'static str {
        "igromania"
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
