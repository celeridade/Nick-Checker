//! Taringa Archived Profile (Blackbird/WMN): cat archived
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: farantic, elrubius
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmTaringaArchivedProfile;

pub const URI: &str = "https://archive.org/wayback/available?url=https://www.taringa.net/{account}";
pub const SAMPLE: Option<&str> = Some("farantic");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"archived_snapshots\": {\"closest\"",
    m_code: 200,
    m_string: "\"archived_snapshots\": {}",
};

#[async_trait]
impl l96a1_core::Platform for WmTaringaArchivedProfile {
    fn key(&self) -> &'static str {
        "wm:taringa-archived-profile"
    }
    fn name(&self) -> &'static str {
        "Taringa Archived Profile"
    }
    fn group(&self) -> &'static str {
        "archived"
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
