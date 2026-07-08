use super::{keyword_chips::KeywordChips, oracle_text::OracleText};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::{
    card::{
        Card,
        scryfall_data::{ImageSize, ScryfallData},
    },
    deck::Board,
};

/// Expandable card row with compact view and optional quantity controls.
#[component]
pub(crate) fn CardRow(
    card: Card,
    qty: i32,
    mut expanded_card: Signal<Option<Uuid>>,
    mut preview_card: Signal<Option<ScryfallData>>,
    mut preview_dismissing: Signal<bool>,
    on_qty_change: Option<EventHandler<i32>>,
    on_move_to: Option<EventHandler<Board>>,
    current_board: Option<Board>,
    on_printing: Option<EventHandler<Card>>,
    /// MVP star state: `Some(true)` filled, `Some(false)` outline, `None` = no
    /// star rendered (non-mainboard rows, tokens, command zone).
    mvp: Option<bool>,
    on_toggle_mvp: Option<EventHandler<()>>,
) -> Element {
    let card_id = card.scryfall_data.id;
    let is_expanded = expanded_card() == Some(card_id);
    let sd = &card.scryfall_data;

    let name = sd.name.clone();
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
    // Color identity is an unordered set; sort to canonical WUBRG order
    // (Color's Ord follows its WUBRG variant declaration).
    let mut colors = sd.color_identity.iter().copied().collect::<Vec<_>>();
    colors.sort();
    let color_codes = colors
        .iter()
        .map(|c| c.to_short_name().to_lowercase())
        .collect::<Vec<_>>();
    let type_line = sd.type_line.clone().unwrap_or_default();
    let keywords = sd.keywords.clone().unwrap_or_default();
    let oracle_text = sd.oracle_text.clone().unwrap_or_default();
    let mana_cost = sd.mana_cost.clone().unwrap_or_default();
    let loyalty_display = sd.loyalty.clone().unwrap_or_default();
    let rarity_name = sd.rarity.to_long_name();
    let has_image: bool = sd.primary_image_url(ImageSize::Large).is_some();
    let scryfall_data_for_preview = sd.clone();
    // Always mounted; the `.open` class drives the grid-rows + opacity collapse
    // so the detail eases open and closed instead of popping.
    let collapse_class = if is_expanded {
        "card-row-collapse open"
    } else {
        "card-row-collapse"
    };

    rsx! {
        div {
            key: "{card_id}",
            class: if is_expanded { "card-row expanded" } else { "card-row" },

            div {
                class: "card-row-compact",
                onclick: move |_| {
                    if expanded_card() == Some(card_id) {
                        expanded_card.set(None);
                    } else {
                        expanded_card.set(Some(card_id));
                    }
                },
                span { class: "card-row-arrow", "▸" }
                span { class: "card-row-qty", "{qty}" }
                span { class: "card-row-name",
                    // MVP star: indicator only, rendered solely on starred
                    // rows (an outline star on every row is 97% noise) —
                    // toggling lives on the expanded view's Star button.
                    if mvp == Some(true) {
                        span { class: "card-row-mvp", "★" }
                    }
                    "{name}"
                }
                span { class: "card-row-cmc", "{cmc_display}" }
                span { class: "card-row-pt", "{pt_display}" }
                span { class: "card-row-colors",
                    for code in color_codes.iter() {
                        i { key: "{code}", class: "ms ms-{code} ms-cost ms-shadow" }
                    }
                }
            }

            div { class: "{collapse_class}",
                div { class: "card-row-collapse-inner",
                hr { class: "card-row-rule" }
                div { class: "card-row-detail",
                    div { class: "card-detail-head",
                        p { class: "card-detail-name", "{name}" }
                        if !mana_cost.is_empty() {
                            OracleText { text: mana_cost, class: "card-detail-cost".to_string() }
                        }
                    }
                    div { class: "card-detail-meta",
                        if !type_line.is_empty() {
                            span { class: "detail-chip", "{type_line}" }
                        }
                        span { class: "detail-chip", "{rarity_name}" }
                    }
                    if !keywords.is_empty() {
                        KeywordChips { keywords }
                    }
                    if !oracle_text.is_empty() {
                        OracleText { text: oracle_text, class: "card-detail-oracle".to_string() }
                    }
                    if !pt_display.is_empty() {
                        div { class: "card-detail-stats",
                            span { class: "detail-chip", "{pt_display}" }
                        }
                    } else if !loyalty_display.is_empty() {
                        div { class: "card-detail-stats",
                            span { class: "detail-chip", "Loyalty {loyalty_display}" }
                        }
                    }
                }
                hr { class: "card-row-rule card-row-rule-muted" }
                div { class: "card-row-actions",
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
                        if has_image {
                            button {
                                class: "qty-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    preview_card.set(Some(scryfall_data_for_preview.clone()));
                                    preview_dismissing.set(false);
                                },
                                "Image"
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
                                        "Printing"
                                    }
                                }
                            }
                        }
                        if let (Some(is_mvp), Some(handler)) = (mvp, on_toggle_mvp) {
                            button {
                                class: "qty-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(());
                                },
                                if is_mvp { "Unstar" } else { "Star" }
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
                                        "To maybeboard"
                                    }
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                                        "To sideboard"
                                    }
                                },
                                Board::Maybeboard => rsx! {
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                                        "To mainboard"
                                    }
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                                        "To sideboard"
                                    }
                                },
                                Board::Sideboard => rsx! {
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                                        "To mainboard"
                                    }
                                    button {
                                        class: "qty-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                                        "To maybeboard"
                                    }
                                },
                            }
                        }
                    }
                }
                hr { class: "card-row-rule card-row-rule-bottom" }
                }
            }
        }
    }
}
