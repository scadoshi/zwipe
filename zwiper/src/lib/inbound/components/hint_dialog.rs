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

/// Hint dialog shell: title, body content, and a single "Got it" button.
/// Compose the body from [`HintLine`]s, with [`HintKey`]s for button names.
/// `dividers` draws a rule under the title and above the button.
#[component]
pub fn HintDialog(
    open: Signal<bool>,
    title: String,
    #[props(default = false)] dividers: bool,
    children: Element,
) -> Element {
    let rule = "border: none; border-top: 1px solid var(--border-primary); width: 100%; margin: 0.25rem 0 0.75rem 0;";
    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v| open.set(v),
            AlertDialogContent {
                AlertDialogTitle { "{title}" }
                if dividers {
                    hr { style: "{rule}" }
                }
                AlertDialogDescription {
                    {children}
                }
                if dividers {
                    hr { style: "{rule}" }
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

/// A dimmed, small credit/attribution line for the foot of a hint
/// (e.g. "Recommendations powered by …").
#[component]
pub fn HintCredit(children: Element) -> Element {
    rsx! {
        p {
            style: "margin: 0.25rem 0 0 0; text-align: left; font-size: 0.78rem; color: var(--text-subtle);",
            {children}
        }
    }
}

/// Bulleted list of hint lines.
#[component]
pub fn HintBullets(children: Element) -> Element {
    rsx! {
        ul { style: "margin: 0 0 0.5rem 0; padding-left: 1.2rem; text-align: left;",
            {children}
        }
    }
}

/// One bullet within [`HintBullets`].
#[component]
pub fn HintBullet(children: Element) -> Element {
    rsx! {
        li { style: "margin-bottom: 0.4rem;", {children} }
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
