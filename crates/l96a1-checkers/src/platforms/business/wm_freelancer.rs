//! Freelancer (Blackbird/WMN): cat business
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: desiaunty, creatvmind
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFreelancer;

pub const URI: &str = "https://www.freelancer.com/api/users/0.1/users?usernames%5B%5D={account}&compact=true";
pub const SAMPLE: Option<&str> = Some("desiaunty");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"users\":{\"",
    m_code: 200,
    m_string: "\"users\":{}",
};

#[async_trait]
impl l96a1_core::Platform for WmFreelancer {
    fn key(&self) -> &'static str {
        "wm:freelancer"
    }
    fn name(&self) -> &'static str {
        "Freelancer"
    }
    fn group(&self) -> &'static str {
        "business"
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
