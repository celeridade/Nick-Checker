//! DIBIZ (Blackbird/WMN): cat business
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: fractalhue, rid
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDibiz;

pub const URI: &str = "https://www.dibiz.com/{account}";
pub const SAMPLE: Option<&str> = Some("fractalhue");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Add to contacts</span>",
    m_code: 404,
    m_string: "An Error Has Occurred",
};

#[async_trait]
impl l96a1_core::Platform for WmDibiz {
    fn key(&self) -> &'static str {
        "wm:dibiz"
    }
    fn name(&self) -> &'static str {
        "DIBIZ"
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
