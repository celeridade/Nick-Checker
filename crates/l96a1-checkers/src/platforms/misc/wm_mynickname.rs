//! MyNickname (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: tw1st, 0xDEFACED
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMynickname;

pub const URI: &str = "https://mynickname.com/en/search?q={account}";
pub const SAMPLE: Option<&str> = Some("tw1st");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "nickname found:</h2>",
    m_code: 200,
    m_string: "<h2>Nickname not found. Try searching for similar nicknames.</h2>",
};

#[async_trait]
impl l96a1_core::Platform for WmMynickname {
    fn key(&self) -> &'static str {
        "wm:mynickname"
    }
    fn name(&self) -> &'static str {
        "MyNickname"
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
