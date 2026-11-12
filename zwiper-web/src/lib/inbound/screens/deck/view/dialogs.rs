//! The deck view's dialogs: Clone, Share with Copy and Stop sharing, Clear
//! skips and Delete. The profile header's icons and the Danger zone panel
//! open them; an action in a dialog closes it, as the app does.

use components::{ActionButton, AlertDialog, DialogAction, ShareUrl, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_name;
use zwipe_core::http::contracts::deck::HttpCloneDeck;
use zwipe_ui::screens::deck_view;
use zwipe_ui::telemetry::vocabulary::component;

use crate::domain::deck::share_link;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::router::Route;
use crate::inbound::telemetry;
use crate::outbound::browser;

/// Which of the deck's dialogs is up. `Copy`, so the buttons that open
/// them and the dialogs share it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeckDialogs {
    /// The Clone dialog.
    pub clone: Signal<bool>,
    /// The Share dialog.
    pub share: Signal<bool>,
    /// The Clear skips dialog.
    pub clear: Signal<bool>,
    /// The Delete dialog.
    pub delete: Signal<bool>,
}

/// Every dialog closed.
pub fn use_deck_dialogs() -> DeckDialogs {
    DeckDialogs {
        clone: use_signal(|| false),
        share: use_signal(|| false),
        clear: use_signal(|| false),
        delete: use_signal(|| false),
    }
}

/// The dialogs for `deck_id`. `share_token` is the deck's live link, which
/// Share and Stop sharing update in place, and `deck_name` names it in that
/// link after the signed-in user's username. `delete_body` is the Delete
/// dialog's line, naming the deck.
#[component]
pub fn DeckDialogsHost(
    deck_id: Uuid,
    dialogs: DeckDialogs,
    mut share_token: Signal<Option<Uuid>>,
    deck_name: String,
    clone_name: String,
    delete_body: String,
) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let DeckDialogs {
        clone: clone_open,
        share: share_open,
        clear: clear_open,
        delete: delete_open,
    } = dialogs;

    // The link's owner, read when a link is shown or copied, and the deck's
    // name for a copy made from a handler, which runs after the effect has
    // caught the name up. The dialog itself names the link from `deck_name`.
    let session: Signal<Option<Session>> = use_context();
    let mut name = use_signal(|| deck_name.clone());
    use_effect(use_reactive!(|deck_name| name.set(deck_name)));
    let link = move |deck: &str, token: Uuid| {
        let owner = session
            .peek()
            .as_ref()
            .map_or_default(|s| s.user.username.to_string());
        share_link(&owner, deck, token)
    };
    let copy_link = move |token: Uuid| {
        browser::copy_to_clipboard(&link(&name.peek(), token));
        toast.info(
            "Share link copied".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };
    let create_link = use_callback(move |()| {
        spawn(async move {
            if let Some(shared) = authed
                .run("share_deck", |c, s| async move {
                    c.share_deck(deck_id, &s).await
                })
                .await
            {
                share_token.set(Some(shared.share_token));
                copy_link(shared.share_token);
            }
        });
    });
    let copy_shared = use_callback(move |()| {
        if let Some(token) = *share_token.peek() {
            copy_link(token);
        }
    });
    let stop_share = use_callback(move |()| {
        spawn(async move {
            if authed
                .run("unshare_deck", |c, s| async move {
                    c.unshare_deck(deck_id, &s).await
                })
                .await
                .is_some()
            {
                share_token.set(None);
                toast.info(
                    "Link disabled".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    // Pending skips go out first, so this window's skips clear too.
    let usage = telemetry::use_usage();
    let client: Signal<ZwipeClient> = use_context();
    let clear_skips = use_callback(move |()| {
        spawn(async move {
            telemetry::flush_once(usage, session, client).await;
            if authed
                .in_component(component::MORE_BUTTONS)
                .run("clear_suppressions", |c, s| async move {
                    c.clear_deck_suppressions(deck_id, &s).await
                })
                .await
                .is_some()
            {
                toast.info(
                    "Skips cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    // Decks replaces the gone deck in history, so Back never reopens it.
    let delete = use_callback(move |()| {
        spawn(async move {
            if authed
                .run("delete_deck", |c, s| async move {
                    c.delete_deck(deck_id, &s).await
                })
                .await
                .is_some()
            {
                navigator.replace(Route::DeckList {});
            }
        });
    });

    let (share_body, share_actions) = if share_token().is_some() {
        (
            "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one.",
            vec![
                DialogAction::Dismiss("Close".into()),
                DialogAction::Confirm {
                    label: "Copy link".into(),
                    danger: false,
                    on_confirm: copy_shared,
                },
                DialogAction::Confirm {
                    label: deck_view::STOP_SHARE.into(),
                    danger: true,
                    on_confirm: stop_share,
                },
            ],
        )
    } else {
        (
            "Anyone with the link can view this deck. You can stop sharing at any time.",
            vec![
                DialogAction::Dismiss("Close".into()),
                DialogAction::Confirm {
                    label: "Create link".into(),
                    danger: false,
                    on_confirm: create_link,
                },
            ],
        )
    };

    rsx! {
        AlertDialog {
            open: share_open,
            title: "Share deck",
            actions: share_actions,
            div { "{share_body}" }
            if let Some(token) = share_token() {
                ShareUrl { url: link(&deck_name, token) }
            }
        }
        AlertDialog {
            open: clear_open,
            title: "Clear skips",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_skips,
                },
            ],
            "Skipped cards come back on their own after a week, and removed cards after a month. Clearing brings them all back now. This can't be undone."
        }
        AlertDialog {
            open: delete_open,
            title: "Delete deck",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Delete".into(),
                    danger: true,
                    on_confirm: delete,
                },
            ],
            "{delete_body}"
        }
        CloneDeckDialog { source_deck_id: deck_id, default_name: clone_name, open: clone_open }
    }
}

/// Clone deck: a copy of the deck under a new name, prefilled with
/// `{name} (clone)` on every open. The name is judged like Create's, with
/// its line under the field, and `Save` stays off until it passes; a failure
/// keeps the dialog open.
#[component]
fn CloneDeckDialog(source_deck_id: Uuid, default_name: String, open: Signal<bool>) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let mut name = use_signal(String::new);
    use_effect(move || {
        // Opening and closing both start the form over.
        let _open = open();
        name.set(default_name.clone());
    });
    let blank = use_memo(move || !deck_view::can_clone(&name()));
    let save = ActionButton::new("Save")
        .disabled(blank)
        .busy("Cloning…", move || {
            let new_name = name.peek().trim().to_owned();
            async move {
                let body = HttpCloneDeck {
                    new_name: new_name.clone(),
                };
                let Some(cloned) = authed
                    .run("clone_deck", |c, s| async move {
                        c.clone_deck(source_deck_id, &body, &s).await
                    })
                    .await
                else {
                    return false;
                };
                toast.info(
                    format!("Cloned as \"{new_name}\""),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                navigator.push(Route::ViewDeck {
                    deck_id: cloned.deck_id,
                });
                true
            }
        });
    rsx! {
        AlertDialog {
            open,
            title: "Clone deck",
            actions: vec![DialogAction::Dismiss("Cancel".into()), save.into()],
            "Make an exact copy of your deck under a new name."
            CloneNameInput { name }
        }
    }
}

/// The new name field and its line, its own component so it reads the name
/// where the dialog host draws it.
#[component]
fn CloneNameInput(mut name: Signal<String>) -> Element {
    let error = deck_view::clone_name_error(&name());
    let error_shown = error.is_some();
    rsx! {
        input {
            r#type: "text",
            class: if error_shown { "input input-error" } else { "input" },
            placeholder: "New deck name",
            maxlength: deck_name::MAX_LENGTH.to_string(),
            value: "{name}",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |evt| name.set(evt.value()),
        }
        if let Some(error) = error {
            div { class: "message-error", "{error}" }
        }
    }
}
