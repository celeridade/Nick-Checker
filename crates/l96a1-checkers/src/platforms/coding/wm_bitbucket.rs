//! Bitbucket (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: LaNMaSteR53, osamahalisawi
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmBitbucket;

pub const URI: &str = "https://bitbucket.org/!api/2.0/repositories/{account}?page=1&pagelen=25&sort=-updated_on&q=&fields=-values.owner%2C-values.workspace";
pub const SAMPLE: Option<&str> = Some("LaNMaSteR53");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "full_name",
    m_code: 404,
    m_string: "No workspace with identifier",
};

#[async_trait]
impl l96a1_core::Platform for WmBitbucket {
    fn key(&self) -> &'static str {
        "wm:bitbucket"
    }
    fn name(&self) -> &'static str {
        "Bitbucket"
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
