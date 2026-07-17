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
    card::{Card, search_card::card_filter::price_currency::PriceCurrency},
    deck::{Board, deck_metrics::card_price},
};

use crate::CardDetails;

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
    /// Fires with the new face index when the expanded detail is flipped, so a
    /// host can mirror the shown side (e.g. zite's hover preview + image overlay).
    on_face_change: Option<EventHandler<usize>>,
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
    /// Opt-in: render the card's classification beside the keywords - the coarse
    /// roles it fulfills, each drilling down to its grouped oracle tags, plus an
    /// "Other tags" bucket. Off by default so read-only/embed hosts (e.g. the
    /// portfolio) are unaffected.
    #[props(default)]
    show_classification: bool,
    /// Resolve an oracle tag's description, making exposed tags tappable-to-reveal
    /// their definition. Forwarded to `CardDetails` → `CardRoleChips`.
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. Forwarded to `CardDetails` →
    /// `CardRoleChips`; shows an "Examples" button on an expanded tag.
    #[props(default)]
    on_examples: Option<Callback<String>>,
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
    let loyalty_display = sd.loyalty.clone().unwrap_or_default();
    // The `actions` slot the shared detail hangs its bar on: qty stepper,
    // printing, star, move-to. Image is a `CardDetails` default, not part of it.
    let has_slot_actions = on_qty_change.is_some()
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
                CardDetails {
                    card: card.clone(),
                    show_classification,
                    describe_tag,
                    on_examples,
                    on_image,
                    on_face_change,
                    has_actions: has_slot_actions,
                    actions: rsx! {
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
                    },
                }
                hr { class: "card-row-rule card-row-rule-bottom" }
                }
            }
        }
    }
}
