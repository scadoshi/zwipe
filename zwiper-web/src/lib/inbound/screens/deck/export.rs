//! Export: the deck as a decklist that round-trips through Import, with the
//! boards to include and `Copy`.
//!
//! On a wide window it is a modal over the deck view, the `Export` chips in
//! a narrow column on the left and the decklist filling the right. Copy is
//! the icon in the dock at the bottom right.

use components::icons::COPY;
use components::{Chip, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::deck::Deck;
use zwipe_ui::deck::export::{self, Boards};

use super::skeletons::ChipRowSkeleton;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::modal_screen::{DockButton, ModalScreen};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::router::Route;
use crate::outbound::browser;

/// The Export screen for `deck_id`.
#[component]
pub fn ExportDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let hint_open = use_one_time_hint(OneTimeHint::Export);
    let mut boards = use_signal(Boards::default);
    let deck: Resource<Result<Deck, ClientError>> = use_resource(move || async move {
        authed
            .try_run("load_deck", |c, s| async move {
                c.fetch_deck(deck_id, &s).await
            })
            .await
    });
    let text = use_memo(move || {
        let held = deck.read();
        let loaded = held.as_ref()?.as_ref().ok()?;
        Some(export::text(loaded, boards()))
    });
    let loading = deck.read().is_none();
    let failed = matches!(&*deck.read(), Some(Err(_)));
    let on = boards();
    let copy = use_callback(move |()| {
        if let Some(list) = text() {
            browser::copy_to_clipboard(&list);
            toast.info(
                "Copied to clipboard".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    });
    let dock = DockButton {
        label: "Copy".to_owned(),
        icon: COPY,
        disabled: text().is_none(),
        onclick: copy,
    };
    rsx! {
        ModalScreen {
            title: "Export",
            description: "Copy the deck as a text decklist, board by board",
            hint_open,
            dock,
            class: "web-modal-wide",
            on_close: move |()| {
                navigator.push(Route::ViewDeck { deck_id });
            },
            div { class: "screen-content content-enter web-export",
                div { class: "import-controls",
                    if loading {
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "md", "sm"] }
                    } else {
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Export" }
                            Chip {
                                selected: on.deck,
                                onclick: move |_| boards.write().toggle_deck(),
                                "Deck"
                            }
                            Chip {
                                selected: on.maybe,
                                onclick: move |_| boards.write().toggle_maybe(),
                                "Maybe"
                            }
                            Chip {
                                selected: on.side,
                                onclick: move |_| boards.write().toggle_side(),
                                "Side"
                            }
                        }
                    }
                }
                div { class: "import-scroll",
                    div { class: "container-sm web-export-body",
                        if let Some(list) = text() {
                            label { class: "label", r#for: "export-text", "Decklist" }
                            textarea {
                                id: "export-text",
                                class: "input",
                                style: "width:100%;min-height:16rem;resize:vertical;",
                                readonly: true,
                                value: "{list}",
                            }
                        } else if failed {
                            p { class: "text-muted", "Could not load deck" }
                        } else {
                            div { class: "skeleton-export",
                                div { class: "skeleton-bar skeleton-export-label" }
                                div { class: "skeleton-export-textarea" }
                            }
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Export, open: hint_open }
    }
}
