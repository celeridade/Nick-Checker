//! Blogspot (Blackbird/WMN): cat blog
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: kaikore, kalenderimlek
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmBlogspot;

pub const URI: &str = "https://{account}.blogspot.com/?hl=en-US";
pub const SAMPLE: Option<&str> = Some("kaikore");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "_WidgetManager._Init",
    m_code: 404,
    m_string: ">Blog not found<",
};

#[async_trait]
impl l96a1_core::Platform for WmBlogspot {
    fn key(&self) -> &'static str {
        "wm:blogspot"
    }
    fn name(&self) -> &'static str {
        "Blogspot"
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
