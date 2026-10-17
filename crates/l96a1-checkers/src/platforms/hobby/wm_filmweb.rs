//! Filmweb (Blackbird/WMN): cat hobby
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: MajkelBlack, Marcin_P
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmFilmweb;

pub const URI: &str = "https://www.filmweb.pl/api/v1/users/{account}/id";
pub const SAMPLE: Option<&str> = Some("MajkelBlack");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "\"userId\":",
    m_code: 204,
    m_string: "",
};

#[async_trait]
impl l96a1_core::Platform for WmFilmweb {
    fn key(&self) -> &'static str {
        "wm:filmweb"
    }
    fn name(&self) -> &'static str {
        "Filmweb"
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
