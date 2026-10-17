//! YouNow (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: lydia_tan33, RavJagz
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmYounow;

pub const URI: &str = "https://api.younow.com/php/api/broadcast/info/user={account}";
pub const SAMPLE: Option<&str> = Some("lydia_tan33");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"userId\":",
    m_code: 200,
    m_string: "\"errorMsg\":\"No users found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmYounow {
    fn key(&self) -> &'static str {
        "wm:younow"
    }
    fn name(&self) -> &'static str {
        "YouNow"
    }
    fn group(&self) -> &'static str {
        "social"
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
