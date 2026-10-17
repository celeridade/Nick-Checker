//! PayPal Business (Blackbird/WMN): cat finance
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: chaminajjan, northstarhealth
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmPaypalBusiness;

pub const URI: &str = "https://www.paypal.com/biz/profile-data/{account}";
pub const SAMPLE: Option<&str> = Some("chaminajjan");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"payerId\":",
    m_code: 404,
    m_string: "\"error\":\"RESOURCE_NOT_FOUND\"",
};

#[async_trait]
impl l96a1_core::Platform for WmPaypalBusiness {
    fn key(&self) -> &'static str {
        "wm:paypal-business"
    }
    fn name(&self) -> &'static str {
        "PayPal Business"
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
