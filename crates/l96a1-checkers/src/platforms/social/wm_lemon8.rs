//! Lemon8 (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: dateandtravell, rolinda_reyes
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmLemon8;

pub const URI: &str = "https://www.lemon8-app.com/{account}?region=us";
pub const SAMPLE: Option<&str> = Some("dateandtravell");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "id=\"user-description\"",
    m_code: 404,
    m_string: "class=\"not_found_text\"",
};

#[async_trait]
impl l96a1_core::Platform for WmLemon8 {
    fn key(&self) -> &'static str {
        "wm:lemon8"
    }
    fn name(&self) -> &'static str {
        "Lemon8"
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
