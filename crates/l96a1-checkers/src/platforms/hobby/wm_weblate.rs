//! Weblate (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: login836039, Datura0808
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmWeblate;

pub const URI: &str = "https://hosted.weblate.org/user/{account}/";
pub const SAMPLE: Option<&str> = Some("login836039");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "class=\"user-page text-center\"",
    m_code: 404,
    m_string: "<h2>Page Not Found</h2>",
};

#[async_trait]
impl l96a1_core::Platform for WmWeblate {
    fn key(&self) -> &'static str {
        "wm:weblate"
    }
    fn name(&self) -> &'static str {
        "Weblate"
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
