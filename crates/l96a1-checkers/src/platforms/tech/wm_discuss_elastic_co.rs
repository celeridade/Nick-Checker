//! discuss.elastic.co (Blackbird/WMN): cat tech
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: whoami, johndoe
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDiscussElasticCo;

pub const URI: &str = "https://discuss.elastic.co/u/{account}";
pub const SAMPLE: Option<&str> = Some("whoami");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "<title>  Profile",
    m_code: 404,
    m_string: "Oops!",
};

#[async_trait]
impl l96a1_core::Platform for WmDiscussElasticCo {
    fn key(&self) -> &'static str {
        "wm:discuss.elastic.co"
    }
    fn name(&self) -> &'static str {
        "discuss.elastic.co"
    }
    fn group(&self) -> &'static str {
        "tech"
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
