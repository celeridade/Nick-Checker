//! RuTracker.org (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Rutracker, Feo156
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmRutrackerOrg;

pub const URI: &str = "https://rutracker.org/forum/profile.php?mode=viewprofile&u={account}";
pub const SAMPLE: Option<&str> = Some("Rutracker");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Профиль пользователя",
    m_code: 200,
    m_string: "Пользователь не найден",
};

#[async_trait]
impl l96a1_core::Platform for WmRutrackerOrg {
    fn key(&self) -> &'static str {
        "wm:rutracker.org"
    }
    fn name(&self) -> &'static str {
        "RuTracker.org"
    }
    fn group(&self) -> &'static str {
        "misc"
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
