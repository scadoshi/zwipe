use crate::inbound::components::alert_dialog::{
    AlertDialogActions, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::components::bottom_sheet::BottomSheet;
use crate::inbound::router::Router;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn MoreButtons(
    deck_id: Uuid,
    show_buy_dialog: Signal<bool>,
    show_more_sheet: Signal<bool>,
    show_delete_dialog: Signal<bool>,
    show_clone_dialog: Signal<bool>,
    has_cards: bool,
    tcg_url: Option<String>,
    ck_url: Option<String>,
) -> Element {
    let navigator = use_navigator();

    rsx! {
        AlertDialogRoot {
            open: show_buy_dialog(),
            on_open_change: move |open| show_buy_dialog.set(open),
            AlertDialogContent {
                AlertDialogTitle { "Buy deck" }
                hr { class: "dialog-rule" }
                if tcg_url.is_none() && ck_url.is_none() {
                    AlertDialogDescription { "No buy links available for this deck." }
                }
                if let Some(ref url) = tcg_url {
                    a {
                        class: "btn",
                        href: "{url}",
                        target: "_blank",
                        style: "text-decoration: none; text-align: center;",
                        onclick: move |_| show_buy_dialog.set(false),
                        "TCGplayer ↗"
                    }
                }
                if let Some(ref url) = ck_url {
                    a {
                        class: "btn",
                        href: "{url}",
                        target: "_blank",
                        style: "text-decoration: none; text-align: center;",
                        onclick: move |_| show_buy_dialog.set(false),
                        "Card Kingdom ↗"
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| show_buy_dialog.set(false),
                        "Close"
                    }
                }
            }
        }

        BottomSheet { open: show_more_sheet, title: "More actions",
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::AddDeckCard { deck_id });
                },
                "Add cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::RemoveDeckCard { deck_id });
                },
                "Remove cards"
            }
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ImportDeck { deck_id });
                },
                "Import cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ExportDeck { deck_id });
                },
                "Export cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_buy_dialog.set(true);
                },
                "Buy deck"
            }
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_clone_dialog.set(true);
                },
                "Clone deck"
            }
            button {
                class: "btn btn-danger",
                onclick: move |_| {
                    show_delete_dialog.set(true);
                },
                "Delete deck"
            }
        }
    }
}
