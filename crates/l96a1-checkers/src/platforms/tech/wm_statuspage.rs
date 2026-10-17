//! Statuspage (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: 8713981tpdlg, gaming, coinbase
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmStatuspage;

pub const URI: &str = "https://{account}.statuspage.io/api/v2/status.json";
pub const SAMPLE: Option<&str> = Some("8713981tpdlg");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "updated_at",
    m_code: 302,
    m_string: "<html><body>You are being <a href=\"https://www.statuspage.io\">redirected</a>.</body></html>",
};

#[async_trait]
impl l96a1_core::Platform for WmStatuspage {
    fn key(&self) -> &'static str {
        "wm:statuspage"
    }
    fn name(&self) -> &'static str {
        "Statuspage"
    }
    fn group(&self) -> &'static str {
        "tech"
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
