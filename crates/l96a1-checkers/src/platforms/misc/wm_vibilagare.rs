//! vibilagare (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: lars01, sven
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmVibilagare;

pub const URI: &str = "https://www.vibilagare.se/users/{account}";
pub const SAMPLE: Option<&str> = Some("lars01");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Profil på vibilagare.se",
    m_code: 404,
    m_string: "Sidan hittades inte |",
};

#[async_trait]
impl l96a1_core::Platform for WmVibilagare {
    fn key(&self) -> &'static str {
        "wm:vibilagare"
    }
    fn name(&self) -> &'static str {
        "vibilagare"
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
