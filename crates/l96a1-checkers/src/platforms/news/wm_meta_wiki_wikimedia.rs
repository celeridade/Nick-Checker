//! Meta-Wiki (Wikimedia) (Blackbird/WMN): cat news
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: sector051, webbreacher
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMetaWikiWikimedia;

pub const URI: &str = "https://meta.wikimedia.org/wiki/Special:CentralAuth?target={account}";
pub const SAMPLE: Option<&str> = Some("sector051");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "id='mw-centralauth-info'",
    m_code: 200,
    m_string: "There is no global account for",
};

#[async_trait]
impl l96a1_core::Platform for WmMetaWikiWikimedia {
    fn key(&self) -> &'static str {
        "wm:meta-wiki-(wikimedia)"
    }
    fn name(&self) -> &'static str {
        "Meta-Wiki (Wikimedia)"
    }
    fn group(&self) -> &'static str {
        "news"
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
