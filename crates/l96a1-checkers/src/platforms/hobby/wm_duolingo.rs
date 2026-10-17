//! Duolingo (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: sdfsdf, duolingo
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDuolingo;

pub const URI: &str = "https://www.duolingo.com/2017-06-30/users?username={account}&_=1628308619574";
pub const SAMPLE: Option<&str> = Some("sdfsdf");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "joinedClassroomIds",
    m_code: 200,
    m_string: "\"users\" : []",
};

#[async_trait]
impl l96a1_core::Platform for WmDuolingo {
    fn key(&self) -> &'static str {
        "wm:duolingo"
    }
    fn name(&self) -> &'static str {
        "Duolingo"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
