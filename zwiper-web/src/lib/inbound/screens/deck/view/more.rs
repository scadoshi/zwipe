//! The deck view's `More` menu and its dialogs: Import and Export, the
//! retailer links in place of the app's `Buy deck`, Clone, Share with Copy
//! and Stop share, Clear skips and Delete.
//!
//! Import, Export, a retailer and Clone close the menu; Share, Clear skips
//! and Delete open their dialog over it, and an action in the dialog closes
//! it, as the app does with its sheet.

use components::{ActionButton, AlertDialog, Button, DialogAction, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::http::contracts::deck::HttpCloneDeck;

use crate::domain::deck::share_link;
use crate::inbound::cards::buy_links::MenuBuyLink;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::more_menu::{MoreMenu, MoreMenuState};
use crate::inbound::router::Route;
use crate::outbound::browser;

/// The menu and its dialogs for `deck_id`. `has_cards` turns on Export and
/// the retailers; `buy` holds the `TCGplayer` and `Card Kingdom` links once
/// the deck has loaded. `share_token` is the deck's live link, which Share
/// and Stop share update in place.
#[component]
pub fn DeckMore(
    deck_id: Uuid,
    menu: MoreMenuState,
    has_cards: bool,
    buy: Option<(String, String)>,
    mut share_token: Signal<Option<Uuid>>,
    clone_name: String,
) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let mut open = menu.open;
    let mut share_open = use_signal(|| false);
    let mut clear_open = use_signal(|| false);
    let mut delete_open = use_signal(|| false);
    let mut clone_open = use_signal(|| false);

    let copy_link = move |token: Uuid| {
        browser::copy_to_clipboard(&share_link(token));
        toast.info(
            "Share link copied".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };
    let create_link = use_callback(move |()| {
        open.set(false);
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
        open.set(false);
        if let Some(token) = *share_token.peek() {
            copy_link(token);
        }
    });
    let stop_share = use_callback(move |()| {
        open.set(false);
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
    // The app first sends its pending skips so they clear too; the web
    // records none.
    let clear_skips = use_callback(move |()| {
        open.set(false);
        spawn(async move {
            if authed
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
    let delete = use_callback(move |()| {
        spawn(async move {
            if authed
                .run("delete_deck", |c, s| async move {
                    c.delete_deck(deck_id, &s).await
                })
                .await
                .is_some()
            {
                navigator.push(Route::DeckList {});
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
                    label: "Stop share".into(),
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
    let (tcgplayer, cardkingdom) = match buy.filter(|_| has_cards) {
        Some((tcg, ck)) => (Some(tcg), Some(ck)),
        None => (None, None),
    };

    rsx! {
        MoreMenu { menu, title: "More actions",
            Button {
                onclick: move |_| {
                    open.set(false);
                    navigator.push(Route::ImportDeck { deck_id });
                },
                "Import cards"
            }
            Button {
                disabled: !has_cards,
                onclick: move |_| {
                    open.set(false);
                    navigator.push(Route::ExportDeck { deck_id });
                },
                "Export cards"
            }
            MenuBuyLink { href: tcgplayer, label: "TCGplayer", on_open: move |()| open.set(false) }
            MenuBuyLink { href: cardkingdom, label: "Card Kingdom", on_open: move |()| open.set(false) }
            Button {
                onclick: move |_| {
                    open.set(false);
                    clone_open.set(true);
                },
                "Clone deck"
            }
            Button { onclick: move |_| share_open.set(true), "Share deck" }
            Button { danger: true, onclick: move |_| clear_open.set(true), "Clear skips" }
            Button { danger: true, onclick: move |_| delete_open.set(true), "Delete deck" }
        }
        AlertDialog {
            open: share_open,
            title: "Share deck",
            actions: share_actions,
            "{share_body}"
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
            "Are you sure you want to delete this deck?"
        }
        CloneDeckDialog { source_deck_id: deck_id, default_name: clone_name, open: clone_open }
    }
}

/// Clone deck: a copy of the deck under a new name, prefilled with
/// `{name} (clone)` on every open. `Save` stays off while the name is
/// blank; a failure keeps the dialog open.
#[component]
fn CloneDeckDialog(source_deck_id: Uuid, default_name: String, open: Signal<bool>) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let mut name = use_signal(String::new);
    use_effect(move || {
        if open() {
            name.set(default_name.clone());
        }
    });
    let blank = use_memo(move || name().trim().is_empty());
    let save = ActionButton::new("Save")
        .disabled(blank)
        .busy("Cloning...", move || {
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

/// The new name field, its own component so it reads the name where the
/// dialog host draws it.
#[component]
fn CloneNameInput(mut name: Signal<String>) -> Element {
    rsx! {
        input {
            r#type: "text",
            class: "input",
            placeholder: "New deck name",
            value: "{name}",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |evt| name.set(evt.value()),
        }
    }
}
