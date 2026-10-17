//! Minecraft: mojang lookup — 200+id=taken, 204/404=avail.
//! Research (`research/minecraft.md`): API antiga SEGUE VIVA (confirmado
//! via proxy). Rate agressivo já mitigado (300ms + 16conc — manter).
//! Vigiar sunset p/ `api.minecraftservices.com`; 429 sempre barreira.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome, UsernameValidator};
use std::time::Instant;

pub struct Minecraft;

/// Decisão pura do lookup.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (200, b) if b.contains("\"id\"") => common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative("api", "id")],
        ),
        (204, _) | (404, _) => common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "miss")],
        ),
        (s, b) => common::finish_http(t, s, b, &["\"id\""], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Minecraft {
    fn key(&self) -> &'static str {
        "minecraft"
    }
    fn name(&self) -> &'static str {
        "Minecraft"
    }
    fn group(&self) -> &'static str {
        "gaming"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://namemc.com/profile/{user}")
    }
    fn validator(&self) -> UsernameValidator {
        UsernameValidator::new(3, 16, r"^[a-z0-9_]+$")
    }
    fn min_interval_ms(&self) -> u64 {
        300
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        l96a1_core::PlatformProfile { max_concurrent: 16, needs_proxy: false, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://api.mojang.com/minecraft/profile/lookup/name/{user}");
        match common::get(ctx, &url, &[("Accept", "application/json")]).await {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => decide(s, &b, ms()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    #[test]
    fn id_is_taken() {
        let o = decide(200, r#"{"id" : "069a79f444e94726a5befca90e38aaf5","name" : "Notch"}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn miss_is_available() {
        assert_eq!(decide(204, "", 5).result, CheckResult::Available);
        assert_eq!(decide(404, "Not found", 5).result, CheckResult::Available);
    }
}
