//! USA Life (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: abaynes79, not1face
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmUsaLife;

pub const URI: &str = "https://usa.life/{account}";
pub const SAMPLE: Option<&str> = Some("abaynes79");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Please log in to like, share and comment",
    m_code: 302,
    m_string: "Sorry, page not found",
};

#[async_trait]
impl l96a1_core::Platform for WmUsaLife {
    fn key(&self) -> &'static str {
        "wm:usa-life"
    }
    fn name(&self) -> &'static str {
        "USA Life"
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
