//! lowcygier.pl (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: armin, janek
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmLowcygierPl;

pub const URI: &str = "https://bazar.lowcygier.pl/user/{account}";
pub const SAMPLE: Option<&str> = Some("armin");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Zarejestrowany",
    m_code: 404,
    m_string: "Błąd 404 - Podana strona nie istnieje",
};

#[async_trait]
impl l96a1_core::Platform for WmLowcygierPl {
    fn key(&self) -> &'static str {
        "wm:lowcygier.pl"
    }
    fn name(&self) -> &'static str {
        "lowcygier.pl"
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
