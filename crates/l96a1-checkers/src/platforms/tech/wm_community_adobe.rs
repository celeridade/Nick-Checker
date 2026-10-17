//! Community Adobe (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: test, janet
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmCommunityAdobe;

pub const URI: &str = "https://community.adobe.com/t5/forums/searchpage/tab/user?q={account}";
pub const SAMPLE: Option<&str> = Some("test");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "UserSearchItemContainer",
    m_code: 200,
    m_string: "No search results found.",
};

#[async_trait]
impl l96a1_core::Platform for WmCommunityAdobe {
    fn key(&self) -> &'static str {
        "wm:community-adobe"
    }
    fn name(&self) -> &'static str {
        "Community Adobe"
    }
    fn group(&self) -> &'static str {
        "tech"
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
