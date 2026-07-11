//! Expandable card row: a compact grid line (qty, name, price, color pips)
//! that eases open to the card's full detail (cost, type, rarity, keywords,
//! oracle text, stats) plus an optional action row.
//!
//! One component for every surface: the app passes edit callbacks (quantity,
//! printing, star, move-to-board); a read-only page like zite's shared deck
//! passes only what it supports (e.g. `on_image`) and hover callbacks for its
//! desktop preview. Every action is an `Option` — `None` simply omits the
//! control.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::{
    card::{
        Card, scryfall_data::ImageSize, search_card::card_filter::price_currency::PriceCurrency,
    },
    deck::{Board, deck_metrics::card_price},
};

use crate::{KeywordChips, OracleText};

/// A card's displayable mana cost, falling back to the front face for
/// double-faced layouts whose top-level cost is absent.
fn display_mana_cost(card: &Card) -> String {
    if let Some(cost) = &card.scryfall_data.mana_cost
        && !cost.is_empty()
    {
        return cost.clone();
    }
    card.scryfall_data
        .card_faces
        .as_ref()
        .and_then(|faces| faces.first())
        .map(|face| face.mana_cost.clone())
        .unwrap_or_default()
}

/// Expandable card row with compact view and optional actions.
#[component]
pub fn CardRow(
    card: Card,
    qty: i32,
    mut expanded_card: Signal<Option<Uuid>>,
    /// Image action: the button renders when the card has art and a handler is
    /// provided; what "view image" means (fullscreen preview, overlay) is the
    /// host's business.
    on_image: Option<EventHandler<()>>,
    on_qty_change: Option<EventHandler<i32>>,
    on_move_to: Option<EventHandler<Board>>,
    current_board: Option<Board>,
    on_printing: Option<EventHandler<Card>>,
    /// MVP star state: `Some(true)` filled, `Some(false)` outline, `None` = no
    /// star rendered (non-mainboard rows, tokens, command zone).
    mvp: Option<bool>,
    on_toggle_mvp: Option<EventHandler<()>>,
    /// Cursor enters/leaves the compact row (desktop hover previews). Never
    /// fires on touch devices (no mouseenter), so touch hosts can omit them.
    on_hover_enter: Option<EventHandler<()>>,
    on_hover_leave: Option<EventHandler<()>>,
    /// Currency for the compact-row price, from the deck's price-target
    /// setting. Defaults to USD.
    #[props(default)]
    price_currency: PriceCurrency,
) -> Element {
    let card_id = card.scryfall_data.id;
    let is_expanded = expanded_card() == Some(card_id);
    let sd = &card.scryfall_data;

    let name = sd.name.clone();
    // Compact-row price in the deck's chosen currency (nonfoil→foil fallback);
    // `None` (no price in that currency) omits the tag entirely.
    let price_display = card_price(sd, price_currency).map(|p| price_currency.format_amount(p));
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
    let mana_cost = display_mana_cost(&card);
    let loyalty_display = sd.loyalty.clone().unwrap_or_default();
    let rarity_name = sd.rarity.to_long_name();
    let has_image: bool = sd.primary_image_url(ImageSize::Large).is_some();
    // The muted rule + actions container render only when at least one action
    // will — an actionless row (read-only, imageless) ends at the bottom rule.
    let has_actions = on_qty_change.is_some()
        || (has_image && on_image.is_some())
        || on_printing.is_some()
        || (mvp.is_some() && on_toggle_mvp.is_some())
        || on_move_to.is_some();
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
                onmouseenter: move |_| {
                    if let Some(handler) = on_hover_enter {
                        handler.call(());
                    }
                },
                onmouseleave: move |_| {
                    if let Some(handler) = on_hover_leave {
                        handler.call(());
                    }
                },
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
                // Card-stat tag: P/T (always slashed, e.g. 4/5) for creatures,
                // else a bare loyalty number for planeswalkers.
                if !pt_display.is_empty() {
                    span { class: "card-row-stat", "{pt_display}" }
                } else if !loyalty_display.is_empty() {
                    span { class: "card-row-stat", "{loyalty_display}" }
                }
                if let Some(price) = price_display {
                    span { class: "card-row-price", "{price}" }
                }
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
                if has_actions {
                hr { class: "card-row-rule card-row-rule-muted" }
                div { class: "card-row-actions",
                    div { class: "card-action-row",
                        if let Some(handler) = on_qty_change {
                            button {
                                class: "card-action-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(-1);
                                },
                                "-"
                            }
                            span { class: "card-action-count", "{qty}" }
                            button {
                                class: "card-action-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(1);
                                },
                                "+"
                            }
                        }
                        if let (true, Some(handler)) = (has_image, on_image) {
                            button {
                                class: "card-action-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(());
                                },
                                "Image"
                            }
                        }
                        if let Some(handler) = on_printing {
                            {
                                let card_clone = card.clone();
                                rsx! {
                                    button {
                                        class: "card-action-btn",
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
                                class: "card-action-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(());
                                },
                                if is_mvp { "Unstar" } else { "Star" }
                            }
                        }
                    }
                    if let Some(handler) = on_move_to {
                        div { class: "card-action-row",
                            match current_board.unwrap_or(Board::Deck) {
                                Board::Deck => rsx! {
                                    button {
                                        class: "card-action-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                                        "To maybeboard"
                                    }
                                    button {
                                        class: "card-action-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                                        "To sideboard"
                                    }
                                },
                                Board::Maybeboard => rsx! {
                                    button {
                                        class: "card-action-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                                        "To mainboard"
                                    }
                                    button {
                                        class: "card-action-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Sideboard); },
                                        "To sideboard"
                                    }
                                },
                                Board::Sideboard => rsx! {
                                    button {
                                        class: "card-action-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Deck); },
                                        "To mainboard"
                                    }
                                    button {
                                        class: "card-action-btn",
                                        style: "white-space:nowrap;",
                                        onclick: move |evt| { evt.stop_propagation(); handler.call(Board::Maybeboard); },
                                        "To maybeboard"
                                    }
                                },
                            }
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
