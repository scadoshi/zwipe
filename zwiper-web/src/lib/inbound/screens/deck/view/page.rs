//! The deck on a wide window: the deck page both sites share, editing. The
//! head carries Edit, Clone, Share and the page's `?` beside the name; the
//! controls add Quick add and the Boards row; the bands are the editor's
//! rows, opening inline with their actions as on the phone, the hover
//! preview pinned to the side the cursor is not on; the stats fold under
//! the cards; and the foot holds Warnings, Data and Danger zone. The dock
//! floats add, remove, the filter and undo.

use std::collections::HashSet;

use components::icons::{DockIcon, MINUS, PLUS, UNDO};
use components::{
    Chip, DeckBand, DeckControls, DeckFacts, DeckFold, DeckHead, DeckPage, DeckSection, DockFilter,
    FeaturedCard,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Board, deck_tag_label};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::{buy, deck_cards, filter};

use super::dialogs::DeckDialogs;
use super::panels::{DangerZonePanel, DataPanel};
use super::sections::{
    BudgetBody, CurrencyChips, DistributionsBody, DrawOddsStepper, ManaBody, ProfileActions,
    TagsBody, WarningFixes, WarningsBody, charted,
};
use crate::domain::deck::{has_budget, tag_count};
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{
    CardOverlays, CardOverlaysHost, HoverCard, HoverPreview, HoverPreviewHost, use_card_overlays,
};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::cards::editor::{
    Editor, Listed, RowInfo, RowKind, main_row_of, row_view, use_deck_editor,
};
use crate::inbound::screens::deck::cards::quick_add::QuickAdd;
use crate::inbound::screens::deck::skeletons::DeckCardsSkeleton;

/// The stat panels' fold keys, shut when the page opens.
const FOLDED_STATS: [&str; 5] = ["Budget", "Tags", "Distributions", "Mana", "Draw odds"];

/// The page for `deck_id`. `fixes` are the warnings' fixes; after one the
/// page reloads the deck.
#[component]
pub fn DeckWidePage(
    deck_id: Uuid,
    currency: Signal<PriceCurrency>,
    fixes: WarningFixes,
    dialogs: DeckDialogs,
    on_hint: Callback<()>,
) -> Element {
    let navigator = use_navigator();
    let ed = use_deck_editor(deck_id);
    let Editor {
        deck,
        entries,
        zone,
        profile,
        loaded,
        tokens,
        mut filter,
        group_by,
        mut boards,
        show_tokens,
        show_zone,
        show_art,
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
        on_drop,
        ..
    } = ed;
    let CardStores { undo, .. } = use_context();
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let CardOverlays { mut preview, .. } = overlays;
    // The hover preview pins to the far side of the window, outside the
    // rows, whose hover scale would otherwise pin it to the row.
    let hover: Signal<Option<HoverCard>> = use_signal(|| None);
    use_context_provider(|| HoverPreview(hover));
    let folded: Signal<HashSet<String>> =
        use_signal(|| FOLDED_STATS.into_iter().map(String::from).collect());
    // A fix made at the foot reloads the deck, so the rows follow.
    let after_fix = use_callback(move |()| ed.reload.call(()));
    let fixes = WarningFixes {
        fix_quantity: use_callback(move |change: (Uuid, i32)| {
            fixes.fix_quantity.call(change);
            after_fix.call(());
        }),
        clear_commander: use_callback(move |()| {
            fixes.clear_commander.call(());
            after_fix.call(());
        }),
        remove: use_callback(move |card: Uuid| {
            fixes.remove.call(card);
            after_fix.call(());
        }),
    };

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
    let held_profile = profile();
    let format = held_profile.as_ref().and_then(|p| p.format);
    let oathbreaker = format.is_some_and(|f| f.has_signature_spell());
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
    let featured = deck_cards::featured(&zone.read(), &entries.read(), oathbreaker);
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
    let has_partner = zone_rows.iter().any(|(slot, _)| *slot == Slot::Partner);

    // The sections, in the editor's order: tokens, the maybeboard and
    // sideboard, the command zone, the main board's groups and its lands.
    let mut sections: Vec<(DeckSection, Vec<RowInfo>, Option<Board>)> = Vec::new();
    if !token_rows.is_empty() {
        let rows = token_rows
            .iter()
            .map(|card| RowInfo {
                card: card.clone(),
                qty: 1,
                kind: RowKind::Token,
            })
            .collect();
        sections.push((
            DeckSection {
                key: "Tokens".to_owned(),
                title: format!("Tokens ({})", token_rows.len()),
                pips: None,
                cards: token_rows,
            },
            rows,
            None,
        ));
    }
    for (board, title, entries_on) in [
        (Board::Maybeboard, "Maybeboard", maybe_rows),
        (Board::Sideboard, "Sideboard", side_rows),
    ] {
        if entries_on.is_empty() {
            continue;
        }
        let rows = entries_on
            .iter()
            .map(|entry| RowInfo {
                card: entry.card.clone(),
                qty: *entry.deck_card.quantity,
                kind: RowKind::Board(board),
            })
            .collect();
        sections.push((
            DeckSection {
                key: title.to_owned(),
                title: format!("{title} ({})", entries_on.len()),
                pips: None,
                cards: entries_on.into_iter().map(|e| e.card).collect(),
            },
            rows,
            Some(board),
        ));
    }
    let commanders: Vec<(Slot, Card)> = zone_rows
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Commander | Slot::Partner))
        .cloned()
        .collect();
    if !commanders.is_empty() {
        let title = Slot::Commander.group_header(format, has_partner).to_owned();
        sections.push((
            DeckSection {
                key: title.clone(),
                title,
                pips: None,
                cards: commanders.iter().map(|(_, c)| c.clone()).collect(),
            },
            commanders
                .into_iter()
                .map(|(slot, card)| RowInfo {
                    card,
                    qty: 1,
                    kind: RowKind::Zone(slot),
                })
                .collect(),
            None,
        ));
    }
    for (slot, card) in zone_rows
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Background | Slot::SignatureSpell))
        .cloned()
    {
        let title = slot.group_header(format, has_partner).to_owned();
        sections.push((
            DeckSection {
                key: title.clone(),
                title,
                pips: None,
                cards: vec![card.clone()],
            },
            vec![RowInfo {
                card,
                qty: 1,
                kind: RowKind::Zone(slot),
            }],
            None,
        ));
    }
    for group in main_groups {
        let n = qty_count(&group.cards);
        let title = if group.label.is_empty() {
            format!("({n})")
        } else {
            format!("{} ({n})", group.label)
        };
        let rows = group
            .cards
            .iter()
            .map(|c| main_row_of(&row_meta, c))
            .collect();
        sections.push((
            DeckSection {
                key: group.key,
                title,
                pips: group.pips,
                cards: group.cards,
            },
            rows,
            Some(Board::Deck),
        ));
    }
    if !main_lands.is_empty() {
        let n = qty_count(&main_lands);
        let rows = main_lands
            .iter()
            .map(|c| main_row_of(&row_meta, c))
            .collect();
        sections.push((
            DeckSection {
                key: "Lands".to_owned(),
                title: format!("Lands ({n})"),
                pips: None,
                cards: main_lands,
            },
            rows,
            Some(Board::Deck),
        ));
    }
    let empty = loaded() && shown_boards.main && sections.is_empty();

    // The fold-all button: every band shut, or every one open again.
    let section_keys: Vec<String> = sections.iter().map(|(s, _, _)| s.key.clone()).collect();
    let all_folded = {
        let shut = collapsed.read();
        !section_keys.is_empty() && section_keys.iter().all(|k| shut.contains(k))
    };
    let fold_all = move |()| {
        let mut collapsed = collapsed;
        let mut shut = collapsed.write();
        if all_folded {
            for key in &section_keys {
                shut.remove(key);
            }
        } else {
            shut.extend(section_keys.iter().cloned());
        }
    };

    // The owner's facts and stats from the deck as fetched.
    let held_deck = deck();
    let metrics = held_deck
        .as_ref()
        .filter(|d| !d.entries.is_empty() || !d.command_zone_cards.is_empty())
        .map(|d| DeckMetrics::from_entries_and_command_zone(&d.entries, &d.command_zone_cards));
    let warnings = held_deck
        .as_ref()
        .map(|d| d.warnings.clone())
        .unwrap_or_default();
    let buy_links = held_deck.as_ref().map(buy::links);
    let has_cards = metrics.is_some();
    let shown = currency();
    let price = metrics.as_ref().and_then(|m| match shown {
        PriceCurrency::Eur => m.total_price_eur,
        PriceCurrency::Tix => m.total_price_tix,
        PriceCurrency::Usd => m.total_price_usd,
    });
    let facts = held_profile.as_ref().map(|p| DeckFacts {
        cards: metrics.as_ref().map_or_else(
            || p.card_count,
            |m| i64::try_from(m.total_cards).unwrap_or(p.card_count),
        ),
        format: p.format,
        power_level: p.power_level,
        zone_names: [
            p.commander_name.clone(),
            p.partner_commander_name.clone(),
            p.background_name.clone(),
            p.signature_spell_name.clone(),
        ]
        .into_iter()
        .flatten()
        .collect(),
        tags: p.tags.iter().map(|t| deck_tag_label(t)).collect(),
        other_tags: p
            .other_tags
            .iter()
            .map(|t| t.display_name().to_owned())
            .collect(),
        price: price.filter(|v| *v > 0.0).map(|v| shown.format_amount(v)),
    });
    let charts = charted(metrics.as_ref());
    let land_count = metrics.as_ref().map_or(0, |m| m.land_count);
    let land_target = held_profile.as_ref().and_then(|p| p.land_target);

    rsx! {
        // The dock floats outside the page's entrance, whose transform would
        // otherwise pin it to the page instead of the window.
        div { class: "content-enter web-page web-deck-page",
            DeckPage {
                head: rsx! {
                if let (Some(p), Some(facts)) = (held_profile.clone(), facts) {
                    DeckHead {
                        name: p.name.to_string(),
                        facts,
                        has_featured: !featured.is_empty(),
                        actions: rsx! {
                            ProfileActions { deck_id, dialogs, on_hint }
                        },
                        for (card, role) in featured.iter().cloned() {
                            FeaturedCard {
                                key: "{card.scryfall_data.id}-{role}",
                                card,
                                role: role.to_string(),
                                on_open: move |(sd, shown_face)| preview.set(Some((sd, shown_face))),
                            }
                        }
                    }
                }
            },
            controls: rsx! {
                if loaded() {
                    DeckControls {
                        group_by,
                        show_tokens: tokens().is_some_and(|t| !t.is_empty()).then_some(show_tokens),
                        show_zone,
                        show_art,
                        all_folded,
                        on_fold_all: fold_all,
                        on_change: move |()| selected.set(None),
                        leading: rsx! {
                            QuickAdd { deck_id, entries, on_added: move |()| {} }
                        },
                        boards: rsx! {
                            div { class: "chip-row",
                                span { class: "stat-chip stat-chip-p1 chip-row-label", "Boards" }
                                for (label, board, on, enabled) in [
                                    ("Main", Board::Deck, shown_boards.main, true),
                                    ("Maybe", Board::Maybeboard, shown_boards.maybe, has_maybe || shown_boards.maybe),
                                    ("Side", Board::Sideboard, shown_boards.side, has_side || shown_boards.side),
                                ] {
                                    span {
                                        key: "{label}",
                                        class: "web-filter-chip",
                                        ondragover: move |e| e.prevent_default(),
                                        ondrop: move |e| {
                                            e.prevent_default();
                                            on_drop.call(board);
                                        },
                                        Chip {
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
                            }
                        },
                    }
                }
            },
            empty,
            bands: rsx! {
                if !loaded() {
                    DeckCardsSkeleton {}
                }
                for (section, rows, drop_to) in sections {
                    DeckBand {
                        key: "{section.key}",
                        section,
                        folded: collapsed,
                        drop_to,
                        on_drop,
                        for row in rows {
                            {row_view(&ed, cur, &row_meta, row)}
                        }
                    }
                }
            },
            stats: rsx! {
                if let Some(p) = held_profile {
                    if has_budget(&p, has_cards) {
                        DeckFold {
                            fold_key: "Budget",
                            title: "Budget",
                            folded,
                            accessory: has_cards.then(|| rsx! { CurrencyChips { currency } }),
                            BudgetBody { profile: p.clone(), metrics, buy: buy_links, currency }
                        }
                    }
                    if tag_count(&p) > 0 {
                        DeckFold { fold_key: "Tags", title: "Tags", folded,
                            TagsBody { profile: p }
                        }
                    }
                }
                if let Some(m) = charts.clone() {
                    DeckFold { fold_key: "Distributions", title: "Distributions", folded,
                        DistributionsBody { metrics: m }
                    }
                }
                if charts.is_some() || land_target.is_some() {
                    DeckFold { fold_key: "Mana", title: "Mana", folded,
                        ManaBody { charts: charts.clone(), land_count, land_target }
                    }
                }
                if let Some(m) = charts {
                    DeckFold {
                        fold_key: "Draw odds",
                        title: "Draw odds",
                        folded,
                        accessory: rsx! { InfoButton { topic: HintTopic::DrawOdds } },
                        DrawOddsStepper { metrics: m }
                    }
                }
            },
            foot: rsx! {
                div { class: "web-column",
                    if !warnings.is_empty() {
                        div { class: "web-section open warn web-deck-warnings",
                            div { class: "web-section-head",
                                div { class: "web-section-title",
                                    span { class: "settings-list-title", "Warnings" }
                                    span { class: "web-section-badge", "{warnings.len()}" }
                                }
                            }
                            hr { class: "web-section-rule" }
                            WarningsBody { deck_id, warnings, fixes }
                        }
                    }
                    DataPanel { deck_id, has_cards }
                }
                DangerZonePanel { dialogs }
            },
            }
        }
        // The page's actions as one floating icon bar at the bottom right:
        // add and remove, then the filter (with its reset while a filter is
        // on) and undo.
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
            button {
                class: "action-dock-btn",
                "aria-label": "Undo",
                disabled: !undo_ready,
                onclick: move |_| apply_undo.call(()),
                DockIcon { path: UNDO }
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
        HoverPreviewHost { preview: hover }
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
