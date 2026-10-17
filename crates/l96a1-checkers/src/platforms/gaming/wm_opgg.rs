//! OPGG (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: xin, carlos01
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmOpgg;

pub const URI: &str = "https://eune.op.gg/summoners/eune/{account}";
pub const SAMPLE: Option<&str> = Some("xin");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "- Summoner Stats - League of Legends",
    m_code: 200,
    m_string: "Guide - OP.GG",
};

#[async_trait]
impl l96a1_core::Platform for WmOpgg {
    fn key(&self) -> &'static str {
        "wm:opgg"
    }
    fn name(&self) -> &'static str {
        "OPGG"
    }
    fn group(&self) -> &'static str {
        "gaming"
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
