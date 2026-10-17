//! ebay_stores (Blackbird/WMN): cat shopping
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: tactical, tactical-security
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmEbayStores;

pub const URI: &str = "https://www.ebay.com/str/{account}";
pub const SAMPLE: Option<&str> = Some("tactical");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "| eBay Stores</title>",
    m_code: 410,
    m_string: "Sorry, this store was not found.",
};

#[async_trait]
impl l96a1_core::Platform for WmEbayStores {
    fn key(&self) -> &'static str {
        "wm:ebay-stores"
    }
    fn name(&self) -> &'static str {
        "ebay_stores"
    }
    fn group(&self) -> &'static str {
        "shopping"
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
