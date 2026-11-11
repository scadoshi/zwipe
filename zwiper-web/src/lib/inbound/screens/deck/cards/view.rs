//! The deck cards view: every card in the deck, to edit, as the app's list
//! below the breakpoint, rows opening inline, with Quick add, Group by,
//! Boards and Show over the groups and add, remove, the filter and undo as
//! the dock's icons. On a wide window the route is the deck page, which
//! edits too. The engine behind both is `editor`.

use std::collections::HashSet;

use components::icons::{DockIcon, MINUS, PLUS, UNDO};
use components::{Chip, DockFilter, IdentityPips};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Board, deck_tag_label};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::screens::deck_cards::TITLE;
use zwipe_ui::telemetry::vocabulary::{DeckScreen, Screen};
use zwipe_ui::{deck_cards, filter};

use super::editor::{
    Editor, Listed, LoadFailure, RowInfo, RowKind, main_row_of, row_view, use_deck_editor,
};
use super::featured::FeaturedCards;
use super::quick_add::QuickAdd;
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardOverlays, CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::chrome::ghost::GhostText;
use crate::inbound::chrome::media::use_wide;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::skeletons::ChipRowSkeleton;
use crate::inbound::screens::deck::view::ViewDeck;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// A collapsible card group. Its header toggles it from a click, Enter or
/// Space, and a shut group's rows are inert.
#[component]
fn CardGroup(
    title: String,
    section_key: String,
    #[props(default)] pips: Option<Vec<Color>>,
    collapsed: Signal<HashSet<String>>,
    children: Element,
) -> Element {
    telemetry::use_telemetry_screen(Screen::Deck(DeckScreen::CardView));
    let open = !collapsed().contains(&section_key);
    let header_class = if open {
        "card-group-header group-collapsible expanded"
    } else {
        "card-group-header group-collapsible"
    };
    let key_for_keys = section_key.clone();
    rsx! {
        div { class: if open { "card-group row-enter" } else { "card-group row-enter collapsed" },
            div {
                class: header_class,
                role: "button",
                tabindex: "0",
                "aria-expanded": "{open}",
                onclick: move |_| flip(collapsed, &section_key),
                onkeydown: move |evt: KeyboardEvent| {
                    if evt.key() == Key::Enter || evt.key() == Key::Character(" ".into()) {
                        evt.prevent_default();
                        flip(collapsed, &key_for_keys);
                    }
                },
                span { class: "card-row-arrow", "\u{25b8}" }
                if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
                    IdentityPips { colors: pips.clone() }
                }
                "{title}"
            }
            div {
                class: if open { "collapsible open" } else { "collapsible" },
                inert: (!open).then_some(true),
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}

/// Shuts the group under `section_key` when it is open, or opens it.
fn flip(mut collapsed: Signal<HashSet<String>>, section_key: &str) {
    let mut set = collapsed.write();
    if !set.remove(section_key) {
        set.insert(section_key.to_owned());
    }
}

/// The deck cards view for `deck_id`: below the breakpoint the app's list,
/// to edit; on a wide window the deck page, which edits too.
#[component]
pub fn ViewDeckCard(deck_id: Uuid) -> Element {
    let wide = use_wide();
    if wide() {
        return rsx! {
            ViewDeck { deck_id }
        };
    }
    let navigator = use_navigator();
    let ed = use_deck_editor(deck_id, LoadFailure::Toast);
    let Editor {
        entries,
        zone,
        profile,
        loaded,
        mut filter,
        mut group_by,
        mut boards,
        mut show_tokens,
        mut show_zone,
        mut show_art,
        collapsed,
        mut selected,
        printing,
        printing_open,
        mut filter_open,
        hint_open,
        listing,
        meta,
        save_printing,
        apply_undo,
        reset_filter,
        ..
    } = ed;
    let CardStores { undo, .. } = use_context();
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let CardOverlays { mut preview, .. } = overlays;

    let Listed {
        tokens: token_rows,
        maybe: maybe_rows,
        side: side_rows,
        zone: zone_rows,
        groups: main_groups,
        lands: main_lands,
        ..
    } = listing();
    let row_meta = meta();
    let applied = filter.read().clone();
    let shown_boards = boards();
    let format = profile.read().as_ref().and_then(|p| p.format);
    let oathbreaker = format.is_some_and(|f| f.has_signature_spell());
    let main_row = |card: &Card| -> RowInfo { main_row_of(&row_meta, card) };
    let qty_count = |cards: &[Card]| -> i32 {
        cards
            .iter()
            .map(|c| row_meta.get(&c.scryfall_data.id).map_or(1, |m| m.qty))
            .fold(0, i32::saturating_add)
    };
    let has_maybe = entries
        .read()
        .iter()
        .any(|e| e.deck_card.board.is_maybeboard());
    let has_side = entries
        .read()
        .iter()
        .any(|e| e.deck_card.board.is_sideboard());
    let featured = profile.read().as_ref().map_or_else(Vec::new, |_| {
        deck_cards::featured(&zone.read(), &entries.read(), oathbreaker)
    });
    let sources = Sources::deck(
        &entries
            .read()
            .iter()
            .map(|e| e.card.clone())
            .collect::<Vec<_>>(),
    );
    let cur = ed.currency();
    let undo_ready = !undo.read().is_empty(deck_id);
    let filter_on = !applied.is_empty();
    let dot = filter::shows_dot(&applied);
    let has_zone = !zone_rows.is_empty();
    let offers_zone = zone.read().offers_chip(format);
    let has_partner = zone_rows.iter().any(|(slot, _)| *slot == Slot::Partner);

    rsx! {
        ScreenFrame {
            title: TITLE,
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content web-deck-cards-view",
                div { class: "web-deck-cards-controls",
                    if let Some(p) = profile() {
                        div { class: "deck-cards-header row-enter",
                            span { class: "deck-cards-header-name", "{p.name}" }
                            if let Some(fmt) = p.format {
                                span { class: "stat-chip stat-chip-p3", "{fmt.display_name()}" }
                            }
                            if let Some(pl) = p.power_level {
                                span { class: "stat-chip stat-chip-p5", "{pl.display_name()}" }
                            }
                            for tag in p.tags.iter() {
                                span { key: "{tag}", class: "stat-chip stat-chip-muted", "{deck_tag_label(tag)}" }
                            }
                            for tag in p.other_tags.iter() {
                                span { key: "{tag}", class: "stat-chip stat-chip-p5", "{tag.display_name()}" }
                            }
                        }
                    } else if !loaded() {
                        div { class: "deck-cards-header",
                            div { class: "skeleton-bar skeleton-deck-list-title" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                        }
                    }
                    FeaturedCards {
                        cards: featured,
                        loaded: loaded(),
                        on_tap: move |card: Card| preview.set(Some((card.scryfall_data, 0))),
                    }
                    if loaded() {
                        QuickAdd { deck_id, entries, on_added: move |()| {} }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Group by" }
                            for option in GroupByOption::all() {
                                Chip {
                                    key: "{option}",
                                    selected: group_by() == option,
                                    onclick: move |_| {
                                        selected.set(None);
                                        group_by.set(option);
                                    },
                                    "{option}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Boards" }
                            for (label, board, on, enabled) in [
                                ("Main", Board::Deck, shown_boards.main, true),
                                ("Maybe", Board::Maybeboard, shown_boards.maybe, has_maybe || shown_boards.maybe),
                                ("Side", Board::Sideboard, shown_boards.side, has_side || shown_boards.side),
                            ] {
                                Chip {
                                    key: "{label}",
                                    selected: on,
                                    disabled: !enabled,
                                    onclick: move |_| {
                                        boards.write().toggle(board);
                                        selected.set(None);
                                    },
                                    "{label}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
                            Chip { selected: show_tokens(), onclick: move |_| show_tokens.toggle(), "Tokens" }
                            if offers_zone {
                                Chip { selected: show_zone(), onclick: move |_| show_zone.toggle(), "Command zone" }
                            }
                            Chip { selected: show_art(), onclick: move |_| show_art.toggle(), "Art" }
                        }
                    } else {
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label web-ghost-chip", GhostText { chars: 9 } }
                            div { class: "input-clear-wrap web-ghost-field",
                                div {
                                    class: "input input-compact input-placeholder",
                                    style: "line-height: normal;",
                                    GhostText { chars: 20 }
                                }
                            }
                        }
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "lg", "md", "lg"] }
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "md", "sm"] }
                        ChipRowSkeleton { label: "sm", chips: vec!["md", "lg", "sm"] }
                    }
                }
                div { class: "web-deck-cards-list",
                    if !loaded() {
                        DeckCardListGhost {}
                    }
                    if !token_rows.is_empty() {
                        CardGroup {
                            title: format!("Tokens ({})", token_rows.len()),
                            section_key: "Tokens",
                            collapsed,
                            for card in token_rows {
                                {row_view(&ed, cur, &row_meta, RowInfo { card, qty: 1, kind: RowKind::Token })}
                            }
                        }
                    }
                    if !maybe_rows.is_empty() {
                        CardGroup {
                            title: format!("Maybeboard ({})", maybe_rows.len()),
                            section_key: "Maybeboard",
                            collapsed,
                            for entry in maybe_rows {
                                {row_view(&ed, cur, &row_meta, RowInfo { qty: *entry.deck_card.quantity, card: entry.card, kind: RowKind::Board(Board::Maybeboard) })}
                            }
                        }
                    }
                    if !side_rows.is_empty() {
                        CardGroup {
                            title: format!("Sideboard ({})", side_rows.len()),
                            section_key: "Sideboard",
                            collapsed,
                            for entry in side_rows {
                                {row_view(&ed, cur, &row_meta, RowInfo { qty: *entry.deck_card.quantity, card: entry.card, kind: RowKind::Board(Board::Sideboard) })}
                            }
                        }
                    }
                    if has_zone {
                        for (slot, card) in zone_rows.clone() {
                            if slot != Slot::Partner || !zone_rows.iter().any(|(s, _)| *s == Slot::Commander) {
                                CardGroup {
                                    key: "{slot:?}",
                                    title: slot.group_header(format, has_partner).to_string(),
                                    section_key: slot.group_key().to_owned(),
                                    collapsed,
                                    {row_view(&ed, cur, &row_meta, RowInfo { card, qty: 1, kind: RowKind::Zone(slot) })}
                                    if slot == Slot::Commander {
                                        for (partner_slot, partner) in zone_rows.iter().filter(|(s, _)| *s == Slot::Partner).cloned() {
                                            {row_view(&ed, cur, &row_meta, RowInfo { card: partner, qty: 1, kind: RowKind::Zone(partner_slot) })}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    for group in main_groups.clone() {
                        CardGroup {
                            key: "{group.key}",
                            title: if group.label.is_empty() { format!("({})", qty_count(&group.cards)) } else { format!("{} ({})", group.label, qty_count(&group.cards)) },
                            section_key: group.key.clone(),
                            pips: group.pips.clone(),
                            collapsed,
                            for card in group.cards.iter() {
                                {row_view(&ed, cur, &row_meta, main_row(card))}
                            }
                        }
                    }
                    if !main_lands.is_empty() {
                        CardGroup {
                            title: format!("Lands ({})", qty_count(&main_lands)),
                            section_key: "Lands",
                            collapsed,
                            for card in main_lands.iter() {
                                {row_view(&ed, cur, &row_meta, main_row(card))}
                            }
                        }
                    }
                    if shown_boards.main && main_groups.is_empty() && main_lands.is_empty() && loaded() {
                        p { class: "deck-empty",
                            span { class: "chip-note", "No cards" }
                        }
                    }
                }
            }
            // The screen's actions as one floating icon bar at the bottom right:
            // add and remove, then the filter (with its reset while a filter is
            // on) and undo. Back is the header's arrow.
            div { class: "action-dock action-dock-right",
                button {
                    class: "action-dock-btn",
                    "aria-label": "Add",
                    onclick: move |_| {
                        navigator.push(Route::AddDeckCard { deck_id });
                    },
                    DockIcon { path: PLUS }
                }
                button {
                    class: "action-dock-btn",
                    "aria-label": "Remove",
                    onclick: move |_| {
                        navigator.push(Route::RemoveDeckCard { deck_id });
                    },
                    DockIcon { path: MINUS }
                }
                span { class: "action-dock-divider" }
                DockFilter {
                    label: "Filter",
                    dot,
                    active: filter_on,
                    onclick: move |()| filter_open.set(true),
                    on_reset: move |()| reset_filter.call(()),
                    reset_label: "Reset filter",
                }
                // Disabled while there is nothing to undo, so the bar keeps
                // its width.
                button {
                    class: "action-dock-btn",
                    "aria-label": "Undo",
                    "data-action": components::data_action::UNDO,
                    disabled: !undo_ready,
                    onclick: move |_| apply_undo.call(()),
                    DockIcon { path: UNDO }
                }
            }
        }
        CardFilterPanel {
            open: filter_open,
            filter,
            config: SheetConfig { show_format: true, sources, ..SheetConfig::default() },
            on_apply: move |next: CardQueryBuilder| {
                filter.set(next);
                selected.set(None);
            },
        }
        CardOverlaysHost { overlays, on_printing_save: move |_| {} }
        OtagExamplesHost { examples }
        OneTimeHintDialog { hint: OneTimeHint::DeckCards, open: hint_open }
        if let Some((card, _)) = printing() {
            PrintingSheet {
                key: "{card.scryfall_data.id}",
                card,
                open: printing_open,
                on_save: move |picked: Card| save_printing.call(picked),
            }
        }
    }
}

/// The groups' ghost while the deck loads: the Commander group's one row,
/// then a full group, in the groups' own header and the rows' height.
#[component]
fn DeckCardListGhost() -> Element {
    rsx! {
        for (i, (title, rows)) in [(9u8, 1usize), (13, 6)].into_iter().enumerate() {
            div { key: "{i}", class: "card-group",
                div { class: "card-group-header",
                    span { GhostText { chars: title } }
                }
                for r in 0..rows {
                    div { key: "{r}", class: "skeleton-card-row web-ghost-card-row",
                        div { class: "skeleton-bar skeleton-card-thumb" }
                        div { class: "skeleton-bar skeleton-card-bar-row" }
                    }
                }
            }
        }
    }
}
