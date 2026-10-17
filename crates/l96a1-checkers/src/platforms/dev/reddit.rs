//! Reddit: /user/{u}/about.json — 404=avail, id=taken.
//! Research (`research/reddit.md`): rota atual CEGA (403 www+old em tudo).
//! Caminhos: rota residencial/sticky ou OAuth app-only (sem login de user).
//! Até ter rota+fixtures: lógica mantida, sem mudança de classificação.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Reddit;

/// Decisão pura do `about.json`.
fn decide(status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let mut out = match (status, body) {
        (404, _) => common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative("api", "404")],
        ),
        (200, b) if b.contains("\"id\"") => common::finish(
            t,
            vec![l96a1_core::Observation::taken_authoritative("api", "id")],
        ),
        (s, b) => common::finish_http(t, s, b, &["\"id\""], &[]),
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Reddit {
    fn key(&self) -> &'static str {
        "reddit"
    }
    fn name(&self) -> &'static str {
        "Reddit"
    }
    fn group(&self) -> &'static str {
        "dev"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://www.reddit.com/user/{user}/")
    }
    fn profile(&self) -> l96a1_core::PlatformProfile {
        // Rota atual cega sem egress residencial (403 www+old em tudo) —
        // tenta direto, mas declara a dependência p/ o doctor/roteador.
        l96a1_core::PlatformProfile { max_concurrent: 32, needs_proxy: true, sticky_proxy: false }
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://www.reddit.com/user/{user}/about.json");
        match common::get(ctx, &url, &[]).await {
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
        let o = decide(200, r#"{"data":{"id":"t2_1","name":"spez"}}"#, 5);
        assert_eq!(o.result, CheckResult::Taken);
    }

    #[test]
    fn missing_404_is_available() {
        let o = decide(404, "{}", 5);
        assert_eq!(o.result, CheckResult::Available);
    }
}
