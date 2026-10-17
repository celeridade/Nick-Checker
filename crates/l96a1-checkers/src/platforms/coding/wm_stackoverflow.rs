//! StackOverflow (Blackbird/WMN): cat coding
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: vonc, bergi
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmStackoverflow;

pub const URI: &str = "https://api.stackexchange.com/2.3/users?order=desc&sort=name&inname={account}&site=stackoverflow";
pub const SAMPLE: Option<&str> = Some("vonc");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"items\":[{",
    m_code: 200,
    m_string: "\"items\":[]",
};

#[async_trait]
impl l96a1_core::Platform for WmStackoverflow {
    fn key(&self) -> &'static str {
        "wm:stackoverflow"
    }
    fn name(&self) -> &'static str {
        "StackOverflow"
    }
    fn group(&self) -> &'static str {
        "coding"
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
