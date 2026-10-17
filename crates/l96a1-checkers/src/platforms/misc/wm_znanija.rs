//! Znanija (Blackbird/WMN): cat misc
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: ila817674, abduabubakir42
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmZnanija;

pub const URI: &str = "https://znanija.com/graphql/ru?operationName=NickAvailability&query=query%20NickAvailability%28%24nick%3AString%21%29%7BnickAvailability%28nick%3A%24nick%29%7BisAvailable%7D%7D&variables=%7B%22nick%22%3A%22{account}%22%7D";
pub const SAMPLE: Option<&str> = Some("ila817674");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"isAvailable\":false",
    m_code: 200,
    m_string: "\"isAvailable\":true",
};

#[async_trait]
impl l96a1_core::Platform for WmZnanija {
    fn key(&self) -> &'static str {
        "wm:znanija"
    }
    fn name(&self) -> &'static str {
        "Znanija"
    }
    fn group(&self) -> &'static str {
        "misc"
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
