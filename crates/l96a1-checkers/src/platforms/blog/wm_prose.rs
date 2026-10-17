//! Prose (Blackbird/WMN): cat blog
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: endeavorance, overthinkification
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmProse;

pub const URI: &str = "https://prose.astral.camp/{account}/";
pub const SAMPLE: Option<&str> = Some("endeavorance");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "blog-title",
    m_code: 404,
    m_string: "Are you sure it was ever here?",
};

#[async_trait]
impl l96a1_core::Platform for WmProse {
    fn key(&self) -> &'static str {
        "wm:prose"
    }
    fn name(&self) -> &'static str {
        "Prose"
    }
    fn group(&self) -> &'static str {
        "blog"
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
