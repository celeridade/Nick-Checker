//! Internet Archive User Contribution Search (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: brewster_kahle, mubix
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmInternetArchiveUserContributionSearch;

pub const URI: &str = "https://archive.org/advancedsearch.php?q=creator:{account}&output=json";
pub const SAMPLE: Option<&str> = Some("brewster_kahle");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "backup_location",
    m_code: 200,
    m_string: "numFound\":0",
};

#[async_trait]
impl l96a1_core::Platform for WmInternetArchiveUserContributionSearch {
    fn key(&self) -> &'static str {
        "wm:internet-archive-user-contribution-search"
    }
    fn name(&self) -> &'static str {
        "Internet Archive User Contribution Search"
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
