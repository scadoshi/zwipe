//! Expandable card row: a compact line (qty, name, price, color pips) that
//! eases open to [`CardDetails`] plus an action row. Every action is an
//! `Option`; `None` omits the control, so read-only hosts pass only what they
//! support.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::{
    card::{Card, search_card::card_filter::price_currency::PriceCurrency},
    deck::{Board, deck_metrics::card_price},
};

use crate::CardDetails;

/// Expandable card row with an optional action bar.
#[component]
pub fn CardRow(
    card: Card,
    qty: i32,
    mut expanded_card: Signal<Option<Uuid>>,
    /// Renders an Image button when the card has art; what "view image" means
    /// is the host's business.
    on_image: Option<EventHandler<()>>,
    /// Fires with the new face index when the expanded detail is flipped.
    on_face_change: Option<EventHandler<usize>>,
    on_qty_change: Option<EventHandler<i32>>,
    on_move_to: Option<EventHandler<Board>>,
    current_board: Option<Board>,
    on_printing: Option<EventHandler<Card>>,
    /// `Some(true)` filled star, `Some(false)` outline, `None` no star.
    mvp: Option<bool>,
    /// Host buttons appended to the expanded action bar. They must be
    /// `card-action-btn`s that stop propagation themselves.
    #[props(default)]
    extra_actions: Option<Element>,
    on_toggle_mvp: Option<EventHandler<()>>,
    /// Desktop hover previews. Never fire on touch devices.
    on_hover_enter: Option<EventHandler<()>>,
    on_hover_leave: Option<EventHandler<()>>,
    /// Currency for the compact-row price. Defaults to USD.
    #[props(default)]
    price_currency: PriceCurrency,
    /// Render card roles and their oracle tags beside the keywords.
    #[props(default)]
    show_classification: bool,
    /// Resolve an oracle tag's description. Forwarded to [`CardDetails`].
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. Forwarded to [`CardDetails`].
    #[props(default)]
    on_examples: Option<Callback<String>>,
    /// Art-crop thumbnail at the far left. `None` renders no art DOM;
    /// `Some(visible)` keeps it mounted and eases it in and out, so a host
    /// toggle animates instead of clipping.
    #[props(default)]
    show_art: Option<bool>,
) -> Element {
    let card_id = card.scryfall_data.id;
    let is_expanded = expanded_card() == Some(card_id);
    let sd = &card.scryfall_data;

    let name = sd.name.clone();
    let art_url = show_art
        .is_some()
        .then(|| sd.art_crop_url().map(str::to_string))
        .flatten();
    let art_class = if show_art == Some(true) {
        "card-row-art"
    } else {
        "card-row-art art-hidden"
    };
    let price_display = card_price(sd, price_currency).map(|p| price_currency.format_amount(p));
    let pt_display = match (&sd.power, &sd.toughness) {
        (Some(p), Some(t)) => format!("{p}/{t}"),
        _ => String::new(),
    };
    // Color identity is unordered; Color's Ord is WUBRG order.
    let mut colors = sd.color_identity.iter().copied().collect::<Vec<_>>();
    colors.sort();
    let color_codes = colors
        .iter()
        .map(|c| c.to_short_name().to_lowercase())
        .collect::<Vec<_>>();
    let loyalty_display = sd.loyalty.clone().unwrap_or_default();
    // Image is a `CardDetails` default, so it doesn't count here.
    let has_slot_actions = on_qty_change.is_some()
        || on_printing.is_some()
        || (mvp.is_some() && on_toggle_mvp.is_some())
        || on_move_to.is_some()
        || extra_actions.is_some();
    // Always mounted; `.open` drives a CSS collapse so the detail eases rather
    // than pops.
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
                if let Some(url) = art_url {
                    img {
                        class: "{art_class}",
                        src: "{url}",
                        loading: "lazy",
                        draggable: false,
                    }
                }
                span { class: "card-row-arrow", "▸" }
                span { class: "card-row-qty", "{qty}" }
                span { class: "card-row-name",
                    // Indicator only; toggling lives on the expanded view's Star button.
                    if mvp == Some(true) {
                        span { class: "card-row-mvp", "★" }
                    }
                    "{name}"
                }
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
                            {extra_actions}
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
