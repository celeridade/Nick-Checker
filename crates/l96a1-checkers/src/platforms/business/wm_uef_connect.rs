//! UEF CONNECT (Blackbird/WMN): cat business
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: heli.mutanen, mette.heiskanen
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmUefConnect;

pub const URI: &str = "https://uefconnect.uef.fi/en/{account}/";
pub const SAMPLE: Option<&str> = Some("heli.mutanen");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "profile-page-header__info",
    m_code: 404,
    m_string: "<title>Page not found - UEFConnect</title>",
};

#[async_trait]
impl l96a1_core::Platform for WmUefConnect {
    fn key(&self) -> &'static str {
        "wm:uef-connect"
    }
    fn name(&self) -> &'static str {
        "UEF CONNECT"
    }
    fn group(&self) -> &'static str {
        "business"
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
