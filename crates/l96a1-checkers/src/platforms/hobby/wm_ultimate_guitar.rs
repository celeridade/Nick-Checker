//! Ultimate Guitar (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: LYNX-Music, Mikhailo, MeGaDeth2314
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmUltimateGuitar;

pub const URI: &str = "https://www.ultimate-guitar.com/u/{account}";
pub const SAMPLE: Option<&str> = Some("LYNX-Music");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: " | Ultimate-Guitar.Com</title>",
    m_code: 410,
    m_string: "Oops! We couldn't find that page.",
};

#[async_trait]
impl l96a1_core::Platform for WmUltimateGuitar {
    fn key(&self) -> &'static str {
        "wm:ultimate-guitar"
    }
    fn name(&self) -> &'static str {
        "Ultimate Guitar"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
