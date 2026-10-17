//! Untappd (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Welcomenetguy, patpatcarney
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmUntappd;

pub const URI: &str = "https://untappd.com/user/{account}/";
pub const SAMPLE: Option<&str> = Some("Welcomenetguy");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"cont user_profile\"",
    m_code: 404,
    m_string: "class=\"search_404\"",
};

#[async_trait]
impl l96a1_core::Platform for WmUntappd {
    fn key(&self) -> &'static str {
        "wm:untappd"
    }
    fn name(&self) -> &'static str {
        "Untappd"
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
