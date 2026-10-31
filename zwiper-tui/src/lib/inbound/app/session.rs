//! Keeping the session and the version gate up to date, as the app's root does on every screen, Login included.
//!
//! Every [`UPKEEP_INTERVAL`], starting at launch, [`upkeep`] refreshes the session when its access token is close to expiring and asks the server for its minimum client version. A refresh the server rejects ends the session, which returns the app to Login without a word; the version gate opens and closes both ways, and a failed version check changes nothing.

use std::time::Duration;

use zwipe_client::ClientError;
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpAnonymousEvent};
use zwipe_core::version;

use super::Context;
use crate::inbound::screens::{Request, Update, VERSION, login};
use crate::outbound::account::Account;

/// How often the session and the version gate are checked, from the app's `UPKEEP_INTERVAL`.
pub(crate) const UPKEEP_INTERVAL: Duration = Duration::from_secs(60);

/// Refreshes the session if it needs it, then checks the version. Returns whether a session there before has ended, and whether this build is too old, `None` when the server could not say.
pub(crate) async fn upkeep(account: &Account) -> Update {
    let had_session = account.session().await.is_some();
    let refreshed = account.fresh().await;
    let ended = had_session
        && matches!(refreshed, Err(ClientError::Unauthorized(_)))
        && account.session().await.is_none();
    let too_old = account
        .client
        .fetch_min_client_version()
        .await
        .ok()
        .map(|minimum| !version::version_at_least(VERSION, &minimum.min_version));
    Update::Upkept { ended, too_old }
}

/// Applies an upkeep: back to Login when the session ended, and the version gate set to what the server said.
pub(crate) fn upkept(cx: &mut Context<'_>, ended: bool, too_old: Option<bool>) {
    if ended {
        session_ended(cx);
    }
    if let Some(too_old) = too_old {
        cx.shared.update_required = too_old;
    }
}

/// Returns to Login without a word, as the app's auth gate does whenever the session clears, and forgets the saved session. Does nothing when no one is signed in.
pub(crate) fn session_ended(cx: &mut Context<'_>) {
    if cx.shared.user.is_none() {
        return;
    }
    login::signed_out(cx, None);
    cx.request(Request::SignOut);
}

/// Sends an anonymous funnel event, fire and forget.
pub(crate) async fn record(account: &Account, kind: AnonymousEventKind) {
    let event = HttpAnonymousEvent {
        session_id: account.launch_id,
        kind,
    };
    account.client.record_anonymous_event(&event).await.ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::event_loop::harness::Harness;
    use crate::inbound::screens::Screen;
    use crate::test_support::user;

    #[test]
    fn the_version_gate_opens_and_closes_and_a_failed_check_keeps_it() {
        let mut harness = Harness::new();
        harness.apply(Update::Upkept {
            ended: false,
            too_old: Some(true),
        });
        assert!(harness.app.shared.update_required);
        harness.apply(Update::Upkept {
            ended: false,
            too_old: None,
        });
        assert!(harness.app.shared.update_required, "fails open: no change");
        harness.apply(Update::Upkept {
            ended: false,
            too_old: Some(false),
        });
        assert!(!harness.app.shared.update_required);
    }

    #[test]
    fn an_ended_session_returns_to_login_silently_once() {
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.user = Some(user(true));
        harness.apply(Update::Upkept {
            ended: true,
            too_old: None,
        });
        assert!(matches!(harness.screen(), Screen::Login(_)));
        assert!(harness.app.shared.toasts.is_empty(), "no toast");
        assert_eq!(harness.pending(), [Request::FetchCounts, Request::SignOut]);
        harness.clear();
        harness.apply(Update::SessionEnded);
        assert_eq!(harness.pending(), [], "already signed out");
    }
}
