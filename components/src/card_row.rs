//! Expandable card row: a compact grid line (qty, name, price, color pips)
//! that eases open to the card's full detail (cost, type, rarity, keywords,
//! oracle text, stats) plus an optional action row.
//!
//! One component for every surface: the app passes edit callbacks (quantity,
//! printing, star, move-to-board); a read-only page like a shared deck
//! passes only what it supports (e.g. `on_image`). Every action is an
//! `Option`; `None` simply omits the control. Under a screen's
//! [`HoverStack`] a collapsed row shows a mouse its card's image there.

use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Board, deck_metrics};

use crate::card_actions::{CardBoardMoves, CardEditButtons};
use crate::hover_stack::{HeldEntry, HoverStack};
use crate::meld::use_meld;
use crate::{CardDetails, CardProxy};

/// Opens the row's detail, letting go of its hover preview, or shuts it
/// when it is open.
fn toggle(
    mut expanded_card: Signal<Option<Uuid>>,
    card_id: Uuid,
    hover_stack: Option<HoverStack>,
    held: &HeldEntry,
) {
    if expanded_card() == Some(card_id) {
        expanded_card.set(None);
    } else {
        if let (Some(stack), Some(id)) = (hover_stack, held.take()) {
            stack.leave(id);
        }
        expanded_card.set(Some(card_id));
    }
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
    /// Fires with the new face index when the expanded detail is flipped, so a
    /// host can mirror the shown side (e.g. zite's image overlay).
    on_face_change: Option<EventHandler<usize>>,
    /// The face the row shows, when the host keeps it: its art, its hover
    /// preview (swapped in place while the cursor is on the row) and its
    /// detail follow it, and Flip hands the next face to `on_face_change`.
    /// Left out, the row starts on the front and its detail keeps its own.
    #[props(default)]
    face: Option<usize>,
    on_qty_change: Option<EventHandler<i32>>,
    on_move_to: Option<EventHandler<Board>>,
    current_board: Option<Board>,
    on_printing: Option<EventHandler<Card>>,
    /// MVP star state: `Some(true)` filled, `Some(false)` outline, `None` = no
    /// star rendered (non-mainboard rows, tokens, command zone).
    mvp: Option<bool>,
    /// Host-specific buttons appended to the expanded action bar (e.g. the
    /// commander maybeboard's Create deck / Remove pair). The host supplies
    /// `card-action-btn` buttons that stop propagation themselves.
    #[props(default)]
    extra_actions: Option<Element>,
    on_toggle_mvp: Option<EventHandler<()>>,
    /// Cursor enters/leaves the compact row. Never fires on touch devices
    /// (no mouseenter), so touch hosts can omit them. The screen's
    /// [`HoverStack`], when there is one, follows the cursor without them.
    on_hover_enter: Option<EventHandler<()>>,
    on_hover_leave: Option<EventHandler<()>>,
    /// Currency for the compact-row price, from the deck's price-target
    /// setting. Defaults to USD.
    #[props(default)]
    price_currency: PriceCurrency,
    /// Opt-in: render the card's classification beside the keywords: the coarse
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
    /// Opt-in: an art-crop thumbnail at the far left of the compact row
    /// (lazy-loaded; rows without an art crop render as before). `None`
    /// (default) renders no art DOM at all, so existing hosts are unaffected;
    /// `Some(visible)` keeps the thumb mounted and eases it in/out, so a host
    /// toggle animates instead of clipping.
    #[props(default)]
    show_art: Option<bool>,
    /// Host content rendered under the expanded detail's chips, before its
    /// action bar. Forwarded to `CardDetails`.
    #[props(default)]
    footer: Option<Element>,
    /// Tags drawn in place of the leading `qty`, each in the P/T tag's
    /// grammar (Playstyle's staples show their deck share there). Empty
    /// (default) keeps the plain qty.
    #[props(default)]
    qty_tags: Vec<String>,
) -> Element {
    let card_id = card.scryfall_data.id;
    // A memo, so the other rows sharing `expanded_card` render again only
    // when their own open state flips.
    let is_expanded = use_memo(use_reactive!(|card_id| expanded_card() == Some(card_id)))();
    // The screen's hover preview: a collapsed row shows the face it shows
    // while the cursor is on it, and lets go on leave, expand or unmount.
    let hover_stack = try_use_context::<HoverStack>();
    let held: HeldEntry = use_hook(|| Rc::new(Cell::new(None)));
    let shown_face: Rc<Cell<usize>> = use_hook(|| Rc::new(Cell::new(0)));
    let held_on_drop = Rc::clone(&held);
    use_drop(move || {
        if let (Some(stack), Some(id)) = (hover_stack, held_on_drop.take()) {
            stack.leave(id);
        }
    });
    let held_on_enter = Rc::clone(&held);
    let held_on_click = Rc::clone(&held);
    let held_on_key = Rc::clone(&held);
    // A meld half with its Meld on shows its melded card in the thumbnail
    // and the hover preview, as everywhere else it shows.
    let meld = use_meld(&card.scryfall_data);
    let melded = meld.shown().is_some();
    let shown = meld
        .shown()
        .cloned()
        .unwrap_or_else(|| card.scryfall_data.clone());
    // The held preview follows a face the host turns, or the half's Meld,
    // while the cursor is on the row.
    let held_on_face = Rc::clone(&held);
    let face_on_change = Rc::clone(&shown_face);
    use_effect(use_reactive!(|face, shown, melded| {
        let face = if melded {
            0
        } else {
            face.unwrap_or_else(|| face_on_change.get())
        };
        if let Some(stack) = hover_stack
            && let Some(url) = shown.shown_face_image_url(face, ImageSize::Normal)
            && let Some(stale) = held_on_face.take()
        {
            held_on_face.set(Some(stack.enter(url.to_string(), shown.name.clone())));
            stack.leave(stale);
        }
    }));
    let held_on_leave = held;
    let face_on_enter = Rc::clone(&shown_face);
    let face_on_flip = shown_face;
    let sd_on_enter = shown.clone();
    let face_change = EventHandler::new(move |face: usize| {
        face_on_flip.set(face);
        if let Some(handler) = on_face_change {
            handler.call(face);
        }
    });
    let sd = &card.scryfall_data;

    let name = sd.name.clone();
    let art_url = show_art
        .is_some()
        .then(|| {
            shown
                .face_art_crop_url(if melded { 0 } else { face.unwrap_or(0) })
                .map(str::to_string)
        })
        .flatten();
    // The art crop that failed to load, which then leaves the card's name in
    // the thumbnail's box.
    let mut art_failed: Signal<Option<String>> = use_signal(|| None);
    let art_url = art_url.filter(|url| art_failed.read().as_deref() != Some(url.as_str()));
    let proxy_sd = (show_art.is_some() && art_url.is_none()).then(|| shown.clone());
    let art_class = if show_art == Some(true) {
        "card-row-art"
    } else {
        "card-row-art art-hidden"
    };
    // Compact-row price: the printing's own price in the deck's currency;
    // `None` (no price in that currency) omits the tag entirely.
    let price_display =
        deck_metrics::card_price(sd, price_currency).map(|p| price_currency.format_amount(p));
    // A multi-faced card keeps its stats on its faces; the row shows the
    // front face's, as its detail opens on it.
    let front = sd.card_faces.as_ref().and_then(|faces| faces.first());
    let power = sd
        .power
        .as_ref()
        .or_else(|| front.and_then(|f| f.power.as_ref()));
    let toughness = sd
        .toughness
        .as_ref()
        .or_else(|| front.and_then(|f| f.toughness.as_ref()));
    let pt_display = match (power, toughness) {
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
    let loyalty_display = sd
        .loyalty
        .clone()
        .or_else(|| front.and_then(|f| f.loyalty.clone()))
        .unwrap_or_default();
    // The `actions` slot the shared detail hangs its bar on: qty stepper,
    // printing, star, move-to. Image is a `CardDetails` default, not part of it.
    let has_slot_actions = on_qty_change.is_some()
        || on_printing.is_some()
        || (mvp.is_some() && on_toggle_mvp.is_some())
        || on_move_to.is_some()
        || extra_actions.is_some();
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
                    if let Some(stack) = hover_stack
                        && expanded_card() != Some(card_id)
                        && let Some(url) = sd_on_enter.shown_face_image_url(if melded { 0 } else { face.unwrap_or_else(|| face_on_enter.get()) }, ImageSize::Normal)
                        && let Some(stale) = held_on_enter.replace(Some(stack.enter(url.to_string(), sd_on_enter.name.clone())))
                    {
                        stack.leave(stale);
                    }
                    if let Some(handler) = on_hover_enter {
                        handler.call(());
                    }
                },
                onmouseleave: move |_| {
                    if let (Some(stack), Some(id)) = (hover_stack, held_on_leave.take()) {
                        stack.leave(id);
                    }
                    if let Some(handler) = on_hover_leave {
                        handler.call(());
                    }
                },
                role: "button",
                tabindex: "0",
                "aria-expanded": "{is_expanded}",
                onclick: move |_| toggle(expanded_card, card_id, hover_stack, &held_on_click),
                onkeydown: move |evt: KeyboardEvent| {
                    if evt.key() == Key::Enter || evt.key() == Key::Character(" ".into()) {
                        evt.prevent_default();
                        toggle(expanded_card, card_id, hover_stack, &held_on_key);
                    }
                },
                if let Some(url) = art_url {
                    // The name sits beside the art, so the art has no alt of its own.
                    img {
                        class: "{art_class}",
                        src: "{url}",
                        loading: "lazy",
                        draggable: false,
                        alt: "",
                        onerror: move |_| art_failed.set(Some(url.clone())),
                    }
                } else if let Some(sd) = proxy_sd {
                    CardProxy { sd, face: face.unwrap_or(0), class: "{art_class}" }
                }
                // The name's head and its tags flow beside the art; the tags
                // wrap under the head when the line is short (see the CSS).
                div { class: "card-row-body",
                    div { class: "card-row-head",
                        span { class: "card-row-arrow", "▸" }
                        if qty_tags.is_empty() {
                            span { class: "card-row-qty", "{qty}" }
                        }
                        for tag in qty_tags {
                            span { class: "card-row-stat", "{tag}" }
                        }
                        span { class: "card-row-name",
                            // MVP star: indicator only, rendered solely on starred
                            // rows (an outline star on every row is 97% noise).
                            // Toggling lives on the expanded view's Star button.
                            if mvp == Some(true) {
                                span { class: "card-row-mvp", "★" }
                            }
                            "{name}"
                        }
                    }
                    // The color identity, the price, then the stat: P/T
                    // (always slashed, e.g. 4/5) for a creature, else a bare
                    // loyalty number for a planeswalker.
                    div { class: "card-row-tags",
                        span { class: "card-row-colors",
                            for code in color_codes.iter() {
                                i { key: "{code}", class: "ms ms-{code} ms-cost ms-shadow" }
                            }
                        }
                        if let Some(price) = price_display {
                            span { class: "card-row-price", "{price}" }
                        }
                        if !pt_display.is_empty() {
                            span { class: "card-row-stat", "{pt_display}" }
                        } else if !loyalty_display.is_empty() {
                            span { class: "card-row-stat", "{loyalty_display}" }
                        }
                    }
                }
            }

            // A shut detail is inert, so Tab and screen readers pass over it.
            div { class: "{collapse_class}", inert: (!is_expanded).then_some(true),
                div { class: "card-row-collapse-inner",
                hr { class: "card-row-rule" }
                CardDetails {
                    card: card.clone(),
                    show_classification,
                    describe_tag,
                    on_examples,
                    on_image,
                    on_face_change: Some(face_change),
                    face,
                    footer,
                    has_actions: has_slot_actions,
                    actions: rsx! {
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
                                            "Printings"
                                        }
                                    }
                                }
                            }
                            CardEditButtons { qty, on_qty_change, mvp, on_toggle_mvp }
                            {extra_actions}
                    },
                    action_rows: rsx! {
                        if let Some(handler) = on_move_to {
                            CardBoardMoves { current_board, on_move_to: handler }
                        }
                    },
                }
                hr { class: "card-row-rule card-row-rule-bottom" }
                }
            }
        }
    }
}
