//! dot.cards (Blackbird/WMN): cat business
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: dakmusic, jhartwell
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDotCards;

pub const URI: &str = "https://dot.cards/{account}";
pub const SAMPLE: Option<&str> = Some("dakmusic");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "status\": \"success",
    m_code: 200,
    m_string: "status\": \"username_not_found",
};

#[async_trait]
impl l96a1_core::Platform for WmDotCards {
    fn key(&self) -> &'static str {
        "wm:dot.cards"
    }
    fn name(&self) -> &'static str {
        "dot.cards"
    }
    fn group(&self) -> &'static str {
        "business"
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
