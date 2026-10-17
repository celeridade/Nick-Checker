//! Dailymotion (Blackbird/WMN): cat video
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: chinesedramastudio, franceinter
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmDailymotion;

pub const URI: &str = "https://api.dailymotion.com/user/{account}?fields=id,username,screenname,description,avatar_720_url,cover_250_url,followers_total,following_total,videos_total,country,created_time,verified,url";
pub const SAMPLE: Option<&str> = Some("chinesedramastudio");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"id\":",
    m_code: 404,
    m_string: "\"code\":404",
};

#[async_trait]
impl l96a1_core::Platform for WmDailymotion {
    fn key(&self) -> &'static str {
        "wm:dailymotion"
    }
    fn name(&self) -> &'static str {
        "Dailymotion"
    }
    fn group(&self) -> &'static str {
        "video"
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
