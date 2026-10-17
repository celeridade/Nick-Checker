//! Telegram: t.me — title + foto decidem (botão existe nos dois estados!).
//! Achado de research (`research/telegram.md`): nick livre TAMBÉM tem
//! `tgme_page_action`/`Send Message` — marcador único antigo = falso Taken.
//! Taken: `<title>Telegram: View @nick</title>` + foto + subscribers.
//! Avail: `<title>Telegram: Contact @nick</title>` (self-ref!) + sem foto.

use crate::common;
use async_trait::async_trait;
use l96a1_core::{CheckCtx, CheckOutcome};
use std::time::Instant;

pub struct Telegram;

/// Decisão pura. `user` entra nos marcadores self-ref.
/// Herdada por `mf:telegram`/`wm:telegram`.
pub(crate) fn decide(user: &str, status: u16, body: &str, ms: u64) -> CheckOutcome {
    let t = Instant::now();
    let lower = body.to_lowercase();
    let u = user.to_lowercase();
    let mut out = if status == 200
        && lower.contains(&format!("<title>telegram: view @{u}</title>"))
        && lower.contains("tgme_page_photo_image")
    {
        common::finish(
            t,
            vec![
                l96a1_core::Observation::taken_authoritative("title", "view-selfref"),
                l96a1_core::Observation::taken_strong("page", "photo"),
            ],
        )
    } else if lower.contains(&format!("<title>telegram: contact @{u}</title>"))
        && !lower.contains("tgme_page_photo_image")
    {
        common::finish(
            t,
            vec![l96a1_core::Observation::avail_authoritative(
                "title",
                "contact-selfref",
            )],
        )
    } else {
        // Título irreconhecível / flood-wall sem marcador: Unknown honesto.
        common::finish(t, vec![])
    };
    out.latency_ms = ms;
    out
}

#[async_trait]
impl l96a1_core::Platform for Telegram {
    fn key(&self) -> &'static str {
        "telegram"
    }
    fn name(&self) -> &'static str {
        "Telegram"
    }
    fn group(&self) -> &'static str {
        "social"
    }
    fn profile_url(&self, user: &str) -> String {
        format!("https://t.me/{user}")
    }
    async fn check(&self, user: &str, ctx: &CheckCtx) -> CheckOutcome {
        let t = Instant::now();
        let ms = || t.elapsed().as_millis() as u64;
        let url = format!("https://t.me/{user}");
        match common::get(ctx, &url, &[]).await {
            None => common::outcome_net_err(ms()),
            Some((s, b)) => decide(user, s, &b, ms()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l96a1_core::CheckResult;

    const TAKEN: &str = "<title>Telegram: View @durov</title>\
        <img class=\"tgme_page_photo_image\" src=\"x\">\
        <div class=\"tgme_page_extra\">10 subscribers</div>";
    const AVAIL: &str = "<title>Telegram: Contact @zzqxkvw123nonexist</title>\
        <a class=\"tgme_action_button_new\" href=\"tg://resolve\">Send Message</a>";

    #[test]
    fn taken_has_title_photo() {
        assert_eq!(decide("durov", 200, TAKEN, 5).result, CheckResult::Taken);
    }

    #[test]
    fn available_despite_action_button() {
        // REGRESSÃO: botão existia nos dois estados e gerava falso Taken.
        assert_eq!(
            decide("zzqxkvw123nonexist", 200, AVAIL, 5).result,
            CheckResult::Available
        );
    }

    #[test]
    fn ambiguous_is_never_taken() {
        let o = decide("x", 200, "<html>weird</html>", 5);
        assert!(o.result == CheckResult::Unknown || o.result == CheckResult::RateLimited);
    }
}
