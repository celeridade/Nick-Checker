//! MyFitnessPal Author (Blackbird/WMN): cat health
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: lauren-krouse, julia-malacoff
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmMyfitnesspalAuthor;

pub const URI: &str = "https://blog.myfitnesspal.com/author/{account}/";
pub const SAMPLE: Option<&str> = Some("lauren-krouse");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "About the Author",
    m_code: 404,
    m_string: "<title>Page not found ",
};

#[async_trait]
impl l96a1_core::Platform for WmMyfitnesspalAuthor {
    fn key(&self) -> &'static str {
        "wm:myfitnesspal-author"
    }
    fn name(&self) -> &'static str {
        "MyFitnessPal Author"
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
