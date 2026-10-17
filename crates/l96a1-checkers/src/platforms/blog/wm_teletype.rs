//! Teletype (Blackbird/WMN): cat blog
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: mart11, anotherj
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmTeletype;

pub const URI: &str = "https://teletype.in/@{account}";
pub const SAMPLE: Option<&str> = Some("mart11");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"layout__content m_main blog\"",
    m_code: 404,
    m_string: "class=\"error\"",
};

#[async_trait]
impl l96a1_core::Platform for WmTeletype {
    fn key(&self) -> &'static str {
        "wm:teletype"
    }
    fn name(&self) -> &'static str {
        "Teletype"
    }
    fn group(&self) -> &'static str {
        "blog"
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
