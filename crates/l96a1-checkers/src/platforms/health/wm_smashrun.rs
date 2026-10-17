//! SmashRun (Blackbird/WMN): cat health
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: john.young
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmSmashrun;

pub const URI: &str = "https://smashrun.com/{account}/";
pub const SAMPLE: Option<&str> = Some("john.young");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "Miles run overall",
    m_code: 404,
    m_string: "no Smashrunner with the username",
};

#[async_trait]
impl l96a1_core::Platform for WmSmashrun {
    fn key(&self) -> &'static str {
        "wm:smashrun"
    }
    fn name(&self) -> &'static str {
        "SmashRun"
    }
    fn group(&self) -> &'static str {
        "health"
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
