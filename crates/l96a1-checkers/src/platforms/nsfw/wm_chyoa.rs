//! Chyoa (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: joe, carlos01
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmChyoa;

pub const URI: &str = "https://chyoa.com/user/{account}";
pub const SAMPLE: Option<&str> = Some("joe");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "When I'm not reading erotica I like to read",
    m_code: 404,
    m_string: "Sorry, I got distracted...",
};

#[async_trait]
impl l96a1_core::Platform for WmChyoa {
    fn key(&self) -> &'static str {
        "wm:chyoa"
    }
    fn name(&self) -> &'static str {
        "Chyoa"
    }
    fn group(&self) -> &'static str {
        "xx NSFW xx"
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
