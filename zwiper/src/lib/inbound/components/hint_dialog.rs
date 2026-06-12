//! One-time hint dialogs.
//!
//! Lightweight, contextual teaching moments: a dialog auto-opens the first
//! time a user reaches a screen (tracked per account via the `hints_shown`
//! map on the user), then never again. Screens may keep a small "?" button
//! that reopens it on demand.

use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogContent, AlertDialogDescription,
    AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::outbound::client::user::mark_hint_shown::ClientMarkHintShown;
use crate::outbound::{client::ZwipeClient, session::Persist};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

/// Opens the returned signal once per account for the given hint key, and
/// reports the hint as shown so it never auto-opens again (on any device).
///
/// Reporting is fire-and-forget: a failed report just means the hint may
/// auto-open once more later. The dialog itself opens optimistically.
pub fn use_one_time_hint(key: &'static str) -> Signal<bool> {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut open = use_signal(|| false);

    use_hook(move || {
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
    });

    open
}

/// Hint dialog shell: title, body content, and a single "Got it" button.
/// Compose the body from [`HintLine`]s, with [`HintKey`]s for button names.
#[component]
pub fn HintDialog(open: Signal<bool>, title: String, children: Element) -> Element {
    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v| open.set(v),
            AlertDialogContent {
                AlertDialogTitle { "{title}" }
                AlertDialogDescription {
                    {children}
                }
                AlertDialogActions {
                    AlertDialogAction {
                        on_click: move |_| open.set(false),
                        "Got it"
                    }
                }
            }
        }
    }
}

/// One body line of a hint dialog.
#[component]
pub fn HintLine(children: Element) -> Element {
    rsx! {
        p { style: "margin: 0 0 0.5rem 0; text-align: left;", {children} }
    }
}

/// An inert reference to an on-screen button: styled like one (util-btn look,
/// accent color) so users recognize what to press, deliberately not tappable
/// since the hint is pointing at the real button, not replacing it.
#[component]
pub fn HintKey(children: Element) -> Element {
    rsx! {
        span {
            style: "border: 1px solid var(--accent-tertiary); color: var(--accent-tertiary); border-radius: 0.5rem; padding: 0.05rem 0.45rem; font-size: 0.8rem; white-space: nowrap; pointer-events: none;",
            {children}
        }
    }
}
