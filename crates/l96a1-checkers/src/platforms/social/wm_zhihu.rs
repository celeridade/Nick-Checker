//! zhihu (Blackbird/WMN): cat social
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: lushnis, kan-shu-jiao-hua-shai-tai-yang
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmZhihu;

pub const URI: &str = "https://api.zhihu.com/books/people/{account}/publications?offset=0&limit=5";
pub const SAMPLE: Option<&str> = Some("lushnis");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"is_start\": true",
    m_code: 404,
    m_string: "\"name\": \"NotFoundException\"",
};

#[async_trait]
impl l96a1_core::Platform for WmZhihu {
    fn key(&self) -> &'static str {
        "wm:zhihu"
    }
    fn name(&self) -> &'static str {
        "zhihu"
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
