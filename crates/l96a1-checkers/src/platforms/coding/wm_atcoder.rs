//! AtCoder (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: apiad, kotatsugame
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmAtcoder;

pub const URI: &str = "https://atcoder.jp/users/{account}";
pub const SAMPLE: Option<&str> = Some("apiad");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<h3>Contest Status</h3>",
    m_code: 404,
    m_string: ">404 Page Not Found</h1>",
};

#[async_trait]
impl l96a1_core::Platform for WmAtcoder {
    fn key(&self) -> &'static str {
        "wm:atcoder"
    }
    fn name(&self) -> &'static str {
        "AtCoder"
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
