use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;

/// Expandable card row with compact view and optional quantity controls.
#[component]
pub(crate) fn CardRow(
    card: Card,
    qty: i32,
    mut expanded_card: Signal<Option<Uuid>>,
    mut preview_image_url: Signal<Option<String>>,
    mut preview_dismissing: Signal<bool>,
    on_qty_change: Option<EventHandler<i32>>,
    on_move_to: Option<EventHandler<Board>>,
    current_board: Option<Board>,
    on_printing: Option<EventHandler<Card>>,
) -> Element {
    let card_id = card.scryfall_data.id;
    let is_expanded = expanded_card() == Some(card_id);
    let sd = &card.scryfall_data;

    let name = sd.name.to_lowercase();
    let cmc_display = sd
        .cmc
        .map(|c| {
            let floored = c.floor() as i64;
            if c == c.floor() {
                format!("{floored}")
            } else {
                format!("{c}")
            }
        })
        .unwrap_or_default();
    let pt_display = match (&sd.power, &sd.toughness) {
        (Some(p), Some(t)) => format!("{p}/{t}"),
        _ => String::new(),
    };
    let color_display = sd
        .color_identity
        .iter()
        .map(|c| format!("{{{}}}", c.to_short_name()))
        .collect::<Vec<_>>()
        .join("");
    let oracle_text = sd.oracle_text.clone().unwrap_or_default().to_lowercase();
    let type_line = sd.type_line.clone().unwrap_or_default().to_lowercase();
    let rarity_name = sd.rarity.to_long_name().to_lowercase();
    let set_name = sd.set_name.clone().to_lowercase();
    let image_url: Option<String> = sd.image_uris.as_ref().and_then(|iu| iu.large.clone());

    rsx! {
        div {
            key: "{card_id}",
            class: if is_expanded { "card-row expanded" } else { "card-row" },
            onclick: move |_| {
                if expanded_card() == Some(card_id) {
                    expanded_card.set(None);
                } else {
                    expanded_card.set(Some(card_id));
                }
            },

            div { class: "card-row-compact",
                span { class: "card-row-qty", "{qty}" }
                span { class: "card-row-name", "{name}" }
                span { class: "card-row-cmc", "{cmc_display}" }
                span { class: "card-row-pt", "{pt_display}" }
                span { class: "card-row-colors", "{color_display}" }
            }

            if is_expanded {
                div { class: "card-row-detail",
                    p { style: "margin-bottom:0.35rem;word-break:break-word;white-space:normal;", "{name}" }
                    if !type_line.is_empty() {
                        span { class: "opacity-50", style: "display:block;margin-bottom:0.5rem;", "{type_line}" }
                    }
                    if !oracle_text.is_empty() {
                        p { class: "card-detail-oracle", "{oracle_text}" }
                    }
                    div { class: "card-detail-meta",
                        span { "{rarity_name} | {set_name}" }
                    }
                    div { class: "qty-row",
                        if let Some(handler) = on_qty_change {
                            button {
                                class: "qty-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(-1);
                                },
                                "-"
                            }
                            span { class: "qty-label", "{qty}" }
                            button {
                                class: "qty-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(1);
                                },
                                "+"
                            }
                        }
                        if let Some(url) = image_url {
                            button {
                                class: "qty-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    preview_image_url.set(Some(url.clone()));
                                    preview_dismissing.set(false);
                                },
                                "image"
                            }
                        }
                        if let Some(handler) = on_printing {
                            {
                                let card_clone = card.clone();
                                rsx! {
                                    button {
                                        class: "qty-btn",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            handler.call(card_clone.clone());
                                        },
                                        "printing"
                                    }
                                }
                            }
                        }
                    }
                    if let Some(handler) = on_move_to {
                        div { class: "qty-row",
                            match current_board.unwrap_or(Board::Deck) {
                                Board::Deck => rsx! {
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                                        "to maybeboard"
                                    }
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                                        "to sideboard"
                                    }
                                },
                                Board::Maybeboard => rsx! {
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                                        "to deck"
                                    }
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                                        "to sideboard"
                                    }
                                },
                                Board::Sideboard => rsx! {
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                                        "to deck"
                                    }
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                                        "to maybeboard"
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
