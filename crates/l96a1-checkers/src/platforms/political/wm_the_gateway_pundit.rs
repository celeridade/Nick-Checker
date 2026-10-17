//! The Gateway Pundit (Blackbird/WMN): cat political
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: andrewmeyer, jimhoft
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmTheGatewayPundit;

pub const URI: &str = "https://www.thegatewaypundit.com/author/{account}/";
pub const SAMPLE: Option<&str> = Some("andrewmeyer");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "meta property=\"og:type\" content=\"profile\"",
    m_code: 200,
    m_string: "<title>The Gateway Pundit * Conservative news and commentary",
};

#[async_trait]
impl l96a1_core::Platform for WmTheGatewayPundit {
    fn key(&self) -> &'static str {
        "wm:the-gateway-pundit"
    }
    fn name(&self) -> &'static str {
        "The Gateway Pundit"
    }
    fn group(&self) -> &'static str {
        "political"
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
