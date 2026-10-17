//! Donatik (Blackbird/WMN): cat finance
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: gypsyjitsu, forestgamp3021
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDonatik;

pub const URI: &str = "https://{account}.donatik.ua/";
pub const SAMPLE: Option<&str> = Some("gypsyjitsu");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\\\"__typename\\\":\\\"UserPageOutput\\\"",
    m_code: 500,
    m_string: "id=\"__next_error__\"",
};

#[async_trait]
impl l96a1_core::Platform for WmDonatik {
    fn key(&self) -> &'static str {
        "wm:donatik"
    }
    fn name(&self) -> &'static str {
        "Donatik"
    }
    fn group(&self) -> &'static str {
        "finance"
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
