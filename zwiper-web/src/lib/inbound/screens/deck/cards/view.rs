//! The deck cards view: every card in the deck, to edit, as the app's List
//! or Stacks below the breakpoint, with Quick add, Group by, Boards, Show and
//! View over the groups and add, remove, the filter and undo as the dock's
//! icons. List rows open inline under group headers with the group's price
//! and cart; the stacks take taps on a touch screen and hover under a mouse. On a wide window the route is the deck page, which
//! edits too. The engine behind both is `editor`.

use std::collections::{HashMap, HashSet};

use components::icons::{DockIcon, MINUS, PLUS, UNDO};
use components::{
    CardGroup, Chip, DeckRowHost, DeckStackDrawer, DeckStacks, DockFilter, GroupBuyDialog,
    StackDetails, TailPlace, deck_sections, group_tail, use_group_buy, use_pointer,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Board, deck_tag_label};
use zwipe_ui::deck_cards::FeaturedOpen;
use zwipe_ui::deck_cards::stacks::{CardsView, Reveal};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::screens::deck_cards::TITLE;
use zwipe_ui::telemetry::vocabulary::{DeckScreen, Screen};
use zwipe_ui::{deck_cards, filter};

use super::editor::{Editor, LoadFailure, row_view, use_deck_editor, use_deck_row_host};
use super::featured::FeaturedCards;
use super::quick_add::QuickAdd;
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardOverlays, CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::stack_view::{
    CardsViewSwitch, use_cards_view, use_stack_details_sync, use_stacks_dealing,
};
use crate::inbound::chrome::ghost::GhostText;
use crate::inbound::chrome::media::use_wide;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::skeletons::ChipRowSkeleton;
use crate::inbound::screens::deck::view::ViewDeck;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

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
    telemetry::use_telemetry_screen(Screen::Deck(DeckScreen::CardView));
    let navigator = use_navigator();
    // Before the editor, whose `i` opens the image preview.
    let overlays = use_card_overlays();
    // The stacks' reveal and the open card, before the editor, whose `d` moves them as a tap.
    let reveal = use_signal(Reveal::default);
    let details_open = use_signal(|| false);
    let details = use_context_provider(|| StackDetails {
        reveal,
        details_open,
    });
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
    let row_host = use_deck_row_host(&ed);
    let CardOverlays { mut preview, .. } = overlays;

    // List or Stacks, as last left, and the stacks in tap mode on a touch screen.
    let view = use_cards_view();
    let stacks_on = view() == CardsView::Stacks;
    let pointer = use_pointer();
    let touch = !pointer();
    let dealing = use_stacks_dealing(view, loaded());
    // The store dialog a group's cart opens, in the List or the stacks.
    let group_buy = use_group_buy();
    let listed_ids = use_memo(move || {
        listing()
            .rows
            .iter()
            .map(|r| r.card.scryfall_data.id)
            .collect::<Vec<_>>()
    });
    use_stack_details_sync(reveal, details_open, listed_ids);

    let listed = listing();
    let all_rows = listed.rows.clone();
    let row_meta = meta();
    let row_boards: HashMap<Uuid, Board> = row_meta.iter().map(|(id, m)| (*id, m.board)).collect();
    let applied = filter.read().clone();
    let shown_boards = boards();
    let format = profile.read().as_ref().and_then(|p| p.format);
    let oathbreaker = format.is_some_and(|f| f.has_signature_spell());
    let no_main = listed.groups.is_empty() && listed.lands.is_empty();
    let sections = deck_sections(listed, &row_meta, format);
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
    let offers_zone = zone.read().offers_chip(format);

    rsx! {
        ScreenFrame {
            title: TITLE,
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div {
                class: "screen-content web-deck-cards-view",
                // Any tap while touch stacks show can reveal a card or close one.
                "data-settle": (stacks_on && touch).then_some(""),
                // A tap off every card closes the reveal; a card's own tap stops at the card.
                onclick: move |_| {
                    if touch {
                        details.dismiss();
                    }
                },
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
                        // One tap opens the card's details in the drawer, as the same card in the
                        // List or Stacks does, or its image when no listed card has details.
                        on_tap: move |card: Card| {
                            let id = card.scryfall_data.id;
                            match deck_cards::featured_open(id, &listed_ids.peek()) {
                                FeaturedOpen::Details => details.click(id),
                                FeaturedOpen::Image => preview.set(Some((card.scryfall_data, 0))),
                            }
                        },
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
                            Chip { selected: show_art(), disabled: stacks_on, onclick: move |_| show_art.toggle(), "Art" }
                        }
                        CardsViewSwitch { view, on_change: move |()| selected.set(None) }
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
                        ChipRowSkeleton { label: "sm", chips: vec!["sm", "md"] }
                    }
                }
                div { class: "web-deck-cards-list", "data-settle-scope": "",
                    if !loaded() {
                        DeckCardListGhost {}
                    }
                    if stacks_on {
                        DeckStacks {
                            // A change of input lays the stacks out again for it.
                            key: "{touch}",
                            sections,
                            currency: cur,
                            collapsed,
                            details,
                            faces: ed.faces,
                            group_buy,
                            dealing: dealing(),
                            touch,
                        }
                    } else {
                        for (section, rows) in sections {
                            CardGroup {
                                key: "{section.key}",
                                title: section.title.clone(),
                                pips: section.pips.clone(),
                                expanded: !collapsed.read().contains(&section.key),
                                ontoggle: {
                                    let key = section.key.clone();
                                    move |()| flip(collapsed, &key)
                                },
                                trailing: group_tail(TailPlace::List, section.title, rows.clone(), cur, group_buy),
                                for row in rows {
                                    {row_view(&ed, row_host, cur, &row_meta, row)}
                                }
                            }
                        }
                    }
                    if shown_boards.main && no_main && loaded() {
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
        GroupBuyDialog { group_buy }
        // The drawer a stack card opens with its second tap.
        DeckStackDrawer {
            rows: all_rows,
            boards: row_boards,
            details,
            faces: ed.faces,
            actions: Some(ed.edit_actions()),
            host: DeckRowHost { on_view_printing: None, ..row_host },
        }
        CardOverlaysHost { overlays }
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
