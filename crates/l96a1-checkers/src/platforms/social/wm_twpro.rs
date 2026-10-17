//! twpro (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: kipuron_ghibli, tsukiusa630
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmTwpro;

pub const URI: &str = "https://twpro.jp/{account}/q-data.json";
pub const SAMPLE: Option<&str> = Some("kipuron_ghibli");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: ",200,\"/",
    m_code: 404,
    m_string: ",404,\"/",
};

#[async_trait]
impl l96a1_core::Platform for WmTwpro {
    fn key(&self) -> &'static str {
        "wm:twpro"
    }
    fn name(&self) -> &'static str {
        "twpro"
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
