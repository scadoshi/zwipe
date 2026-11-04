//! One-time hint dialogs.
//!
//! Lightweight, contextual teaching moments: a dialog auto-opens the first
//! time a user reaches a screen (tracked per account via the `hints_shown`
//! map on the user), then never again. Screens may keep a small "?" button
//! that reopens it on demand. What a dialog says is zwipe-ui's
//! [`one_time`](zwipe_ui::hint::one_time) copy, drawn by
//! [`OneTimeHintDialog`](crate::inbound::components::hint_dialog::OneTimeHintDialog).

use components::hint::ContentHintDialog;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::hint::one_time;

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::guide_link;
use crate::outbound::session::Persist;

/// Opens the returned signal once per account for the given hint key, and
/// reports the hint as shown so it never auto-opens again (on any device).
///
/// Fires at mount. For hints gated on async state (e.g. "the deck has
/// cards"), call [`open_and_record_hint`] from an effect instead, once the
/// gate first passes.
pub fn use_one_time_hint(key: &'static str) -> Signal<bool> {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let open = use_signal(|| false);

    use_hook(move || open_and_record_hint(key, session, client, open));

    open
}

/// Opens the dialog and records the hint as shown, unless this user has
/// already seen it. The shared trigger behind [`use_one_time_hint`]; callers
/// with an async gate invoke it directly (guarded so it runs once).
///
/// Reporting is fire-and-forget: a failed report just means the hint may
/// auto-open once more later. The dialog itself opens optimistically.
pub fn open_and_record_hint(
    key: &'static str,
    mut session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    mut open: Signal<bool>,
) {
    let seen = session
        .peek()
        .as_ref()
        .is_none_or(|s| s.user.has_seen_hint(key));
    if seen {
        return;
    }
    open.set(true);
    spawn(async move {
        let Ok(s) = session.ensure_fresh(client).await else {
            return;
        };
        let http = client.peek().clone();
        match http.mark_hint_shown(key, &s).await {
            Ok(fresh_user) => {
                let current = session.peek().clone();
                if let Some(mut current) = current {
                    current.user = fresh_user;
                    // persist so the hint stays seen across app restarts
                    current.infallible_save();
                    session.set(Some(current));
                }
            }
            Err(e) => tracing::warn!("failed to record hint {key}: {e}"),
        }
    });
}

/// The dialog `hint` opens: its copy from [`one_time::content_for`] and its
/// guide link, when zwipe.net has one. Nothing for a hint whose copy is
/// built per overlay, like Swipe select.
#[component]
pub fn OneTimeHintDialog(open: Signal<bool>, hint: OneTimeHint) -> Element {
    let Some(content) = one_time::content_for(hint) else {
        return rsx! {};
    };
    rsx! {
        ContentHintDialog { open, content, actions: guide_link::one_time(hint.key()) }
    }
}
