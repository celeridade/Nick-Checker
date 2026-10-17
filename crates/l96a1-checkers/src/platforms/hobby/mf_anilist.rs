//! Anilist (Sherlock manifest): https://anilist.co/
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Cat: hobby | via: override | NSFW: false | sample: Josh
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct MfAnilist;

pub const URL: &str = "https://anilist.co/user/{}/";
pub const URL_MAIN: &str = "https://anilist.co/";
pub const SAMPLE: Option<&str> = Some("Josh");
pub const BAKED: site_probe::BakedManifest = site_probe::BakedManifest {
    error_type: site_probe::SiteErrorType::Message,
    error_msg: &["\"User\":null", "Not Found"],
    error_code: &[],
    regex: Some("^[A-Za-z0-9]{2,20}$"),
    method: "POST",
    payload: Some("{\"query\": \"query($name:String){User(name:$name){id}}\", \"variables\": {\"name\": \"{}\"}}"),
    headers: &[],
    url_probe: Some("https://graphql.anilist.co/"),
    error_url: None,
};

#[async_trait]
impl l96a1_core::Platform for MfAnilist {
    fn key(&self) -> &'static str {
        "mf:anilist"
    }
    fn name(&self) -> &'static str {
        "Anilist"
    }
    fn group(&self) -> &'static str {
        "hobby"
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
