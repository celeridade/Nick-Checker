//! WordPress.com (Private) (Blackbird/WMN): cat blog
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: webbreacher, mary
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmWordpressComPrivate;

pub const URI: &str = "https://public-api.wordpress.com/rest/v1.1/sites/{account}.wordpress.com";
pub const SAMPLE: Option<&str> = Some("webbreacher");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 403,
    e_string: "\"message\":\"User cannot access this private blog.\"",
    m_code: 404,
    m_string: "\"error\":\"unknown_blog\"",
};

#[async_trait]
impl l96a1_core::Platform for WmWordpressComPrivate {
    fn key(&self) -> &'static str {
        "wm:wordpress.com-(private)"
    }
    fn name(&self) -> &'static str {
        "WordPress.com (Private)"
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
