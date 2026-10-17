//! cafecito (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: braftty, guillermo
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmCafecito;

pub const URI: &str = "https://cafecito.app/{account}";
pub const SAMPLE: Option<&str> = Some("braftty");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: " | Cafecito</title>",
    m_code: 404,
    m_string: "Es posible que el enlace que seleccionaste esté roto o que se haya eliminado la página",
};

#[async_trait]
impl l96a1_core::Platform for WmCafecito {
    fn key(&self) -> &'static str {
        "wm:cafecito"
    }
    fn name(&self) -> &'static str {
        "cafecito"
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
