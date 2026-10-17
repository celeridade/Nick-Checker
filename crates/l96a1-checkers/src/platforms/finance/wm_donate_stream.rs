//! donate.stream (Blackbird/WMN): cat finance
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: requiemzxc_komaru, hexttr
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDonateStream;

pub const URI: &str = "https://donate.stream/api/v1/streamer.get?path={account}&app=9f4e793cec820015d511dbc77b20c5c1";
pub const SAMPLE: Option<&str> = Some("requiemzxc_komaru");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"response\":",
    m_code: 200,
    m_string: "\"message\":\"Not found\"",
};

#[async_trait]
impl l96a1_core::Platform for WmDonateStream {
    fn key(&self) -> &'static str {
        "wm:donate.stream"
    }
    fn name(&self) -> &'static str {
        "donate.stream"
    }
    fn group(&self) -> &'static str {
        "finance"
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
