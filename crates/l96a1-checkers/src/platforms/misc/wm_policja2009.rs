//! policja2009 (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Pvwel, janek
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPolicja2009;

pub const URI: &str = "http://www.policja2009.fora.pl/search.php?search_author={account}";
pub const SAMPLE: Option<&str> = Some("Pvwel");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Autor",
    m_code: 200,
    m_string: "Nie znaleziono tematów ani postów pasujących do Twoich kryteriów",
};

#[async_trait]
impl l96a1_core::Platform for WmPolicja2009 {
    fn key(&self) -> &'static str {
        "wm:policja2009"
    }
    fn name(&self) -> &'static str {
        "policja2009"
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
