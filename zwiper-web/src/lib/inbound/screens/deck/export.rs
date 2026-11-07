//! Export: the deck as a decklist that round-trips through Import, with the
//! boards to include and `Copy`.
//!
//! A screen at every width: the `Export` chips over the decklist. Copy is
//! the icon in the dock at the bottom right.

use components::icons::{COPY, DockIcon};
use components::{Chip, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::deck::Deck;
use zwipe_ui::deck::export::{self, Boards};

use super::skeletons::ChipRowSkeleton;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::ghost::GhostText;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::shell::ScreenFrame;
use crate::outbound::browser;

/// The Export screen for `deck_id`.
#[component]
pub fn ExportDeck(deck_id: Uuid) -> Element {
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
    let on_hint = use_callback(move |()| {
        let mut hint_open = hint_open;
        hint_open.set(true);
    });
    rsx! {
        ScreenFrame {
            title: "Export",
            description: Some("Copy the deck as a text decklist, board by board".to_owned()),
            on_hint,
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
                            div { class: "web-form-panel",
                                div { class: "settings-list-header web-form-head",
                                    span { class: "settings-list-title", "Decklist" }
                                }
                                div { class: "web-form-fields",
                                    textarea {
                                        id: "export-text",
                                        class: "input",
                                        "aria-label": "Decklist",
                                        style: "width:100%;min-height:16rem;resize:vertical;",
                                        readonly: true,
                                        value: "{list}",
                                    }
                                }
                            }
                        } else if failed {
                            p { class: "text-muted", "Could not load deck" }
                        } else {
                            div { class: "web-form-panel",
                                div { class: "settings-list-header web-form-head",
                                    span { class: "settings-list-title", GhostText { chars: 8 } }
                                }
                                div { class: "web-form-fields",
                                    div { class: "skeleton-export-textarea" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "action-dock action-dock-right",
                button {
                    class: "action-dock-btn",
                    "aria-label": "Copy",
                    disabled: text().is_none(),
                    onclick: move |_| copy.call(()),
                    DockIcon { path: COPY }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Export, open: hint_open }
    }
}
