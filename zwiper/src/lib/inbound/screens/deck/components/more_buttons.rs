use crate::inbound::components::bottom_sheet::BottomSheet;
use crate::inbound::router::Router;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub(crate) fn MoreButtons(
    deck_id: Uuid,
    show_buy_sheet: Signal<bool>,
    show_more_sheet: Signal<bool>,
    show_delete_dialog: Signal<bool>,
    has_cards: bool,
    tcg_url: Option<String>,
    ck_url: Option<String>,
) -> Element {
    let navigator = use_navigator();

    rsx! {
        BottomSheet { open: show_buy_sheet, title: "buy deck",
            if let Some(ref url) = tcg_url {
                a {
                    class: "btn",
                    href: "{url}",
                    target: "_blank",
                    style: "text-decoration: none; text-align: center;",
                    onclick: move |_| show_buy_sheet.set(false),
                    "tcgplayer"
                }
            }
            if let Some(ref url) = ck_url {
                a {
                    class: "btn",
                    href: "{url}",
                    target: "_blank",
                    style: "text-decoration: none; text-align: center;",
                    onclick: move |_| show_buy_sheet.set(false),
                    "card kingdom"
                }
            }
        }

        BottomSheet { open: show_more_sheet, title: "more actions",
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::AddDeckCard { deck_id });
                },
                "add cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::RemoveDeckCard { deck_id });
                },
                "remove cards"
            }
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ImportDeck { deck_id });
                },
                "import cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ExportDeck { deck_id });
                },
                "export cards"
            }
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_delete_dialog.set(true);
                },
                "delete deck"
            }
        }
    }
}
