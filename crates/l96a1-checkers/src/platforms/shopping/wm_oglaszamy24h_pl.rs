//! oglaszamy24h.pl (Blackbird/WMN): cat shopping
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: kowal, janek
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmOglaszamy24hPl;

pub const URI: &str = "https://oglaszamy24h.pl/profil,{account}";
pub const SAMPLE: Option<&str> = Some("kowal");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Profil użytkownika:",
    m_code: 404,
    m_string: "Nieprawidłowy link, w bazie danych nie istnieje użytkownik o podanym loginie",
};

#[async_trait]
impl l96a1_core::Platform for WmOglaszamy24hPl {
    fn key(&self) -> &'static str {
        "wm:oglaszamy24h.pl"
    }
    fn name(&self) -> &'static str {
        "oglaszamy24h.pl"
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
