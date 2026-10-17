//! GNOME VCS (Sherlock manifest): https://gitlab.gnome.org/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: coding | via: keyword | NSFW: false | sample: adam
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfGnomeVcs;

pub const URL: &str = "https://gitlab.gnome.org/{}";
pub const URL_MAIN: &str = "https://gitlab.gnome.org/";
pub const SAMPLE: Option<&str> = Some("adam");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::ResponseUrl,
    error_msg: &[],
    error_code: &[],
    regex: Some("^(?!-)[a-zA-Z0-9_.-]{2,255}(?<!\\.)$"),
    method: "GET",
    payload: None,
    headers: &[],
    url_probe: None,
    error_url: Some("https://gitlab.gnome.org/{}"),
};

#[async_trait]
impl l96a1_core::Platform for MfGnomeVcs {
    fn key(&self) -> &'static str {
        "mf:gnome-vcs"
    }
    fn name(&self) -> &'static str {
        "GNOME VCS"
    }
    fn group(&self) -> &'static str {
        "coding"
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 24, needs_proxy: false, sticky_proxy: false }
    }
    fn profile_url(&self, user: &str) -> String {
        site_probe::interpolate(URL, user)
    }
    fn validator(&self) -> UsernameValidator {
        site_probe::validator_for(BAKED.regex)
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        site_probe::probe_manifest(&BAKED, URL, user, ctx).await
    }
}
