//! Choko.Link (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: ForgedSteelTools, eugeniapsychology
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmChokoLink;

pub const URI: &str = "https://choko.link/{account}";
pub const SAMPLE: Option<&str> = Some("ForgedSteelTools");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "--title-font-family: Roboto;",
    m_code: 404,
    m_string: "The link might be incorrect or the profile has been deleted.",
};

#[async_trait]
impl l96a1_core::Platform for WmChokoLink {
    fn key(&self) -> &'static str {
        "wm:choko.link"
    }
    fn name(&self) -> &'static str {
        "Choko.Link"
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
