//! A staple's deck count under its expanded row, and the decks behind it.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::playstyle::StapleDeck;

/// `label` ("4 of 11 decks") as a chip. With `decks` it toggles like a card's
/// "Other tags" chip, easing open one chip per deck, each opening that deck
/// through `on_open`. Without any (an older server sent no deck ids, or no
/// id is in the user's deck list) it is the plain count and does not expand.
#[component]
pub fn StapleDecksChip(
    label: String,
    decks: Vec<StapleDeck>,
    on_open: EventHandler<Uuid>,
) -> Element {
    let mut open = use_signal(|| false);
    if decks.is_empty() {
        return rsx! {
            div { class: "keyword-chips",
                span { class: "detail-chip detail-chip-pt", "{label}" }
            }
        };
    }
    rsx! {
        div { class: "keyword-chips",
            button {
                r#type: "button",
                class: if open() { "chip selected" } else { "chip" },
                "aria-expanded": "{open()}",
                onclick: move |evt| {
                    evt.stop_propagation();
                    open.toggle();
                },
                "{label}"
            }
        }
        // The decks stay mounted while the reveal eases shut.
        div { class: if open() { "keyword-reveal open" } else { "keyword-reveal" },
            div { class: "keyword-reveal-inner",
                div { class: "otag-reveal-block",
                    div { class: "card-detail-meta card-detail-otags staple-decks",
                        for deck in decks {
                            button {
                                key: "{deck.id}",
                                r#type: "button",
                                class: "chip",
                                tabindex: if open() { "0" } else { "-1" },
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    on_open.call(deck.id);
                                },
                                "{deck.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
