//! Salon24 (Blackbird/WMN): cat blog
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: matuzalem, niewiarygodne
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSalon24;

pub const URI: &str = "https://www.salon24.pl/u/{account}/";
pub const SAMPLE: Option<&str> = Some("matuzalem");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<span>Obserwujących</span>",
    m_code: 301,
    m_string: "<title>Salon24 - Blogi, wiadomości, opinie i komentarze",
};

#[async_trait]
impl l96a1_core::Platform for WmSalon24 {
    fn key(&self) -> &'static str {
        "wm:salon24"
    }
    fn name(&self) -> &'static str {
        "Salon24"
    }
    fn group(&self) -> &'static str {
        "blog"
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
