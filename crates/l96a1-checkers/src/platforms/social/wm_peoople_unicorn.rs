//! Peoople (Unicorn) (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: carmensalva, teletextopuntocom
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPeoopleUnicorn;

pub const URI: &str = "https://peoople.app/en/unicorn/{account}/";
pub const SAMPLE: Option<&str> = Some("carmensalva");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"@type\":\"BreadcrumbList\"",
    m_code: 404,
    m_string: "id=\"__next_error__\"",
};

#[async_trait]
impl l96a1_core::Platform for WmPeoopleUnicorn {
    fn key(&self) -> &'static str {
        "wm:peoople-(unicorn)"
    }
    fn name(&self) -> &'static str {
        "Peoople (Unicorn)"
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
