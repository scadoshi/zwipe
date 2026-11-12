//! One-time hints: a dialog that opens by itself the first time a user
//! reaches a screen, on any device, and never again.
//!
//! Whether a user has seen one lives on the server, in the user's
//! `hints_shown` map. The dialog opens at once and the hint is recorded with
//! `mark_hint_shown` in the background; a failed record only means it may
//! open once more later. A screen's header `?` reopens it on demand without
//! recording it again. The hints are zwipe-core's and their copy zwipe-ui's.

use components::DialogAction;
use components::hint::{ContentHintDialog, guide_for_one_time, guide_for_swipe_select};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
pub use zwipe_core::domain::user::models::hints::OneTimeHint;
use zwipe_ui::hint::{SwipeSelectMode, one_time};

use crate::inbound::chrome::shortcuts_sheet::use_shortcuts_action;
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::session_store::SessionStore;

/// Opens the returned signal at mount, once per account, and records the
/// hint. For a hint gated on something that loads later, call
/// [`open_and_record_hint`] once the gate first passes instead.
pub fn use_one_time_hint(hint: OneTimeHint) -> Signal<bool> {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let open = use_signal(|| false);
    use_hook(move || open_and_record_hint(hint, session, client, open));
    open
}

/// Opens `open` and records `hint` as shown, unless there is no session or
/// the user has seen it. A caller with a gate guards it to run once.
pub fn open_and_record_hint(
    hint: OneTimeHint,
    mut session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    mut open: Signal<bool>,
) {
    let key = hint.key();
    let seen = session
        .peek()
        .as_ref()
        .is_none_or(|s| s.user.has_seen_hint(key));
    if seen {
        return;
    }
    open.set(true);
    spawn(async move {
        let Ok(current) = ensure_fresh(session, client).await else {
            return;
        };
        let http = client.peek().clone();
        match http.mark_hint_shown(key, &current).await {
            Ok(user) => {
                let latest = session.peek().clone();
                if let Some(mut latest) = latest {
                    latest.user = user;
                    SessionStore::browser().infallible_save(&latest);
                    session.set(Some(latest));
                }
            }
            Err(e) => tracing::warn!("failed to record hint {key}: {e}"),
        }
    });
}

/// A hint's `actions`, then the keyboard button where the screen offers it
/// ([`use_shortcuts_action`]), so it sits beside the guide link.
fn with_shortcuts(
    mut actions: Vec<DialogAction>,
    shortcuts: Option<DialogAction>,
) -> Vec<DialogAction> {
    actions.extend(shortcuts);
    actions
}

/// The dialog for every one-time hint but [`OneTimeHint::SwipeSelect`],
/// whose copy changes with its overlay: see [`SwipeSelectHint`].
#[component]
pub fn OneTimeHintDialog(hint: OneTimeHint, open: Signal<bool>) -> Element {
    let shortcuts = use_shortcuts_action();
    let Some(content) = one_time::content_for(hint) else {
        return rsx! {};
    };
    rsx! {
        ContentHintDialog { open, content, actions: with_shortcuts(guide_for_one_time(hint), shortcuts) }
    }
}

/// The first-deck hint for the deck page's layout: the wide page's dock or
/// the narrow stack's Profile card and `Cards` pill.
#[component]
pub fn FirstDeckHintDialog(open: Signal<bool>, wide: bool) -> Element {
    let shortcuts = use_shortcuts_action();
    rsx! {
        ContentHintDialog {
            open,
            content: one_time::first_deck(if wide { one_time::DeckLayout::Wide } else { one_time::DeckLayout::Narrow }),
            actions: with_shortcuts(guide_for_one_time(OneTimeHint::FirstDeck), shortcuts),
        }
    }
}

/// The `swipe_select` hint for the overlay's `mode`.
#[component]
pub fn SwipeSelectHint(open: Signal<bool>, mode: SwipeSelectMode) -> Element {
    let shortcuts = use_shortcuts_action();
    rsx! {
        ContentHintDialog {
            open,
            content: one_time::swipe_select(mode),
            actions: with_shortcuts(guide_for_swipe_select(mode), shortcuts),
        }
    }
}
