//! TF2 Backpack Examiner (Blackbird/WMN): cat gaming
//! RESEARCH (preencher a mao): logo _ | avatar _ | banner _ | created _ | email _ | id _
//! Known: Hasnof, McCrasAaronMcCras
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use crate::platforms::site_probe;

pub struct WmTf2BackpackExaminer;

pub const URI: &str = "https://www.tf2items.com/id/{account}";
pub const SAMPLE: Option<&str> = Some("Hasnof");
pub const BAKED: site_probe::BakedWmn = site_probe::BakedWmn {
    e_code: 200,
    e_string: "var profileId",
    m_code: 302,
    m_string: "",
};

#[async_trait]
impl l96a1_core::Platform for WmTf2BackpackExaminer {
    fn key(&self) -> &'static str {
        "wm:tf2-backpack-examiner"
    }
    fn name(&self) -> &'static str {
        "TF2 Backpack Examiner"
    }
    fn group(&self) -> &'static str {
        "gaming"
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
