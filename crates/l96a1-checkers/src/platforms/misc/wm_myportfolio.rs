//! myportfolio (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: artkonina, fox
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMyportfolio;

pub const URI: &str = "https://{account}.myportfolio.com/work";
pub const SAMPLE: Option<&str> = Some("artkonina");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"page-title",
    m_code: 302,
    m_string: "<title>Adobe Portfolio | Build your own personalized website</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmMyportfolio {
    fn key(&self) -> &'static str {
        "wm:myportfolio"
    }
    fn name(&self) -> &'static str {
        "myportfolio"
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
