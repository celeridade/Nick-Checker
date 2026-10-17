//! Joe Monster (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: dandris, lasior
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmJoeMonster;

pub const URI: &str = "https://joemonster.org/bojownik/{account}";
pub const SAMPLE: Option<&str> = Some("dandris");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "jest prywatny",
    m_code: 200,
    m_string: "Nie wiem jak ci to powiedzieć",
};

#[async_trait]
impl l96a1_core::Platform for WmJoeMonster {
    fn key(&self) -> &'static str {
        "wm:joe-monster"
    }
    fn name(&self) -> &'static str {
        "Joe Monster"
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
