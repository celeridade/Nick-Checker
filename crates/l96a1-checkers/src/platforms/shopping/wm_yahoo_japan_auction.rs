//! Yahoo! JAPAN Auction (Blackbird/WMN): cat shopping
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: fltr14502003
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmYahooJapanAuction;

pub const URI: &str = "https://auctions.yahoo.co.jp/follow/list/{account}";
pub const SAMPLE: Option<&str> = Some("fltr14502003");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "出品者",
    m_code: 500,
    m_string: "Yahoo! JAPAN IDが無効です。",
};

#[async_trait]
impl l96a1_core::Platform for WmYahooJapanAuction {
    fn key(&self) -> &'static str {
        "wm:yahoo!-japan-auction"
    }
    fn name(&self) -> &'static str {
        "Yahoo! JAPAN Auction"
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
