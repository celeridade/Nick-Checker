//! Parler archived profile (Blackbird/WMN): cat archived
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: JoePags, dineshdsouza
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmParlerArchivedProfile;

pub const URI: &str = "http://archive.org/wayback/available?url=https://parler.com/profile/{account}";
pub const SAMPLE: Option<&str> = Some("JoePags");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"archived_snapshots\": {\"closest\"",
    m_code: 200,
    m_string: "\"archived_snapshots\": {}",
};

#[async_trait]
impl l96a1_core::Platform for WmParlerArchivedProfile {
    fn key(&self) -> &'static str {
        "wm:parler-archived-profile"
    }
    fn name(&self) -> &'static str {
        "Parler archived profile"
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
