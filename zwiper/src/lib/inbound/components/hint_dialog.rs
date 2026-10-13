//! One-time hint dialogs.
//!
//! Lightweight, contextual teaching moments: a dialog auto-opens the first
//! time a user reaches a screen (tracked per account via the `hints_shown`
//! map on the user), then never again. Screens may keep a small "?" button
//! that reopens it on demand.

use crate::{inbound::components::auth::ensure_session::EnsureFresh, outbound::session::Persist};
use components::{AlertDialog, DialogAction};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;

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

/// Hint dialog shell: title, body content, and a "Got it" button.
/// Compose the body from [`HintLine`](components::HintLine)s, with
/// [`HintKey`](components::HintKey)s for button names.
///
/// `actions` go before "Got it" in the footer (e.g. a "Dictionary" button),
/// keeping call-to-actions out of the body.
#[component]
pub fn HintDialog(
    open: Signal<bool>,
    title: String,
    children: Element,
    #[props(default)] actions: Vec<DialogAction>,
) -> Element {
    let mut actions = actions;
    actions.push(DialogAction::Dismiss("Got it".into()));
    rsx! {
        AlertDialog {
            open,
            title,
            actions,
            div { class: "hint-body", style: "padding: 1rem 0;", {children} }
        }
    }
}

/// A color-coded word inside a hint (e.g. a swipe direction). `color` is a
/// CSS variable name like `--color-success`.
#[component]
pub fn HintColored(color: String, children: Element) -> Element {
    rsx! {
        span { style: "color: var({color}); font-weight: 600;", {children} }
    }
}
