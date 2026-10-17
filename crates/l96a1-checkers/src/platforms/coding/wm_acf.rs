//! ACF (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: mike, greg
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmAcf;

pub const URI: &str = "https://support.advancedcustomfields.com/forums/users/{account}/";
pub const SAMPLE: Option<&str> = Some("mike");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "bbp-user-body",
    m_code: 404,
    m_string: "<title>Page Not Found - ACF Support</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmAcf {
    fn key(&self) -> &'static str {
        "wm:acf"
    }
    fn name(&self) -> &'static str {
        "ACF"
    }
    fn group(&self) -> &'static str {
        "coding"
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
