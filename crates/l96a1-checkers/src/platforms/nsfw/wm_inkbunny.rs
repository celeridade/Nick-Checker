//! InkBunny (Blackbird/WMN): cat xx NSFW xx
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: AdminBunny, test
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmInkbunny;

pub const URI: &str = "https://inkbunny.net/{account}";
pub const SAMPLE: Option<&str> = Some("AdminBunny");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Profile | Inkbunny, the Furry Art Community</title>",
    m_code: 302,
    m_string: "<title>Members | Inkbunny, the Furry Art Community</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmInkbunny {
    fn key(&self) -> &'static str {
        "wm:inkbunny"
    }
    fn name(&self) -> &'static str {
        "InkBunny"
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
