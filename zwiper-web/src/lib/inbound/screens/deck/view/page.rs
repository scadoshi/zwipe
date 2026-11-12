//! The deck on a wide window: the deck page both sites share, editing. The
//! head names the deck over its facts and featured cards, as the shared page
//! does (the page's `?` sits in the nav); the controls
//! add Quick add and the Boards row; the bands are the editor's rows,
//! opening inline with their actions as on the phone, the hover preview
//! pinned to the side the cursor is not on; the stats fold under the cards;
//! and the foot holds Warnings, Data and Danger zone. The dock floats add,
//! remove, the filter and undo, then Edit, Clone and Share.

use std::collections::HashSet;
use std::time::Duration;

use components::icons::{CloneIcon, DockIcon, EditIcon, MINUS, PLUS, ShareIcon, UNDO};
use components::{
    CART, CartButton, Chip, DeckBand, DeckControls, DeckFacts, DeckFold, DeckHead, DeckPage,
    DeckSection, DockFilter, FeaturedCard, HoverStackHost, StoreDialog, use_hover_stack,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Board, deck_tag_label};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::stacks::{
    CardsView, DEAL_DURATION, DEAL_STAGGER_CAP, DealCue, Reveal, deal_delay,
};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::screens::deck_view::{EDIT, MoreItem};
use zwipe_ui::{buy, deck_cards, filter};

use super::dialogs::DeckDialogs;
use super::ghost::{
    DeckBandsGhost, DeckControlsGhost, DeckFootGhost, DeckHeadGhost, DeckStatsGhost,
};
use super::group_buy::{group_buy_dialog, group_cards, use_group_buy};
use super::panels::{DangerZonePanel, DataPanel};
use super::sections::{
    BudgetBody, CurrencyChips, DistributionsBody, DrawOddsStepper, ManaBody, TagsBody,
    WarningFixes, WarningsBody, charted,
};
use super::settle::use_settle;
use super::stacks::{stack_details, stacks_view};
use crate::domain::deck::{has_budget, tag_count};
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardOverlays, CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::stack_view::{
    CardsViewSwitch, StackDetails, use_cards_view, use_stack_details_sync,
};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::cards::editor::{
    Editor, Listed, LoadFailure, RowInfo, RowKind, main_row_of, row_view, use_deck_editor,
};
use crate::inbound::screens::deck::cards::quick_add::QuickAdd;
use crate::outbound::timer;

/// The stat panels' fold keys, shut when the page opens.
const FOLDED_STATS: [&str; 5] = ["Budget", "Tags", "Distributions", "Mana", "Draw odds"];

/// How long the featured cards wait for their images before they deal anyway, as on
/// zwipe.net's shared deck.
const FEATURED_WAIT: Duration = Duration::from_secs(4);

/// The page for `deck_id`. `fixes` are the warnings' fixes, and `fixed`
/// counts the ones that have landed: each reloads the deck.
#[component]
pub fn DeckWidePage(
    deck_id: Uuid,
    currency: Signal<PriceCurrency>,
    fixes: WarningFixes,
    fixed: ReadSignal<u32>,
    dialogs: DeckDialogs,
) -> Element {
    let navigator = use_navigator();
    let DeckDialogs {
        mut clone,
        mut share,
        ..
    } = dialogs;
    // Quiet: the deck view's own profile fetch raises the one toast.
    // Before the editor, whose `i` opens the image preview.
    let overlays = use_card_overlays();
    // The stacks' reveal and the open card, before the editor, whose `d` moves them as a click.
    let reveal = use_signal(Reveal::default);
    let details_open = use_signal(|| false);
    let stack_details_state = use_context_provider(|| StackDetails {
        reveal,
        details_open,
    });
    let ed = use_deck_editor(deck_id, LoadFailure::Quiet);
    use_settle();
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
        ..
    } = ed;
    let CardStores { undo, .. } = use_context();
    let examples = use_otag_reveals();
    let CardOverlays { mut preview, .. } = overlays;
    // The rows' hover stack, drawn outside the rows and the page's entrance,
    // whose transforms would pin it to them instead of the window.
    let hover_stack = use_hover_stack();
    let folded: Signal<HashSet<String>> =
        use_signal(|| FOLDED_STATS.into_iter().map(String::from).collect());
    // List or Stacks, as last left.
    let view = use_cards_view();
    // The stacks deal in on entering Stacks, once the cards are in, for as
    // long as the longest stack takes.
    let mut deal_cue = use_signal(DealCue::default);
    let mut dealing = use_signal(|| false);
    use_effect(move || {
        let shown = view();
        let ready = loaded();
        if deal_cue.write().on(shown, ready) {
            dealing.set(true);
            spawn(async move {
                timer::sleep(deal_delay(DEAL_STAGGER_CAP).saturating_add(DEAL_DURATION)).await;
                dealing.set(false);
            });
        }
    });
    // The store dialog a group's cart opens, in the List or the stacks.
    let group_buy = use_group_buy();
    // The deck's store dialog, from the dock's cart.
    let mut deck_buy_open = use_signal(|| false);
    let listed_ids = use_memo(move || {
        listing()
            .rows
            .iter()
            .map(|r| r.card.scryfall_data.id)
            .collect::<Vec<_>>()
    });
    use_stack_details_sync(reveal, details_open, listed_ids);
    // A fix made at the foot reloads the deck once it has landed, so the
    // rows, the counts and the warnings follow.
    use_effect(move || {
        if fixed() > 0 {
            ed.reload.call(());
        }
    });

    let all_rows = listing().rows;
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
    // The featured cards with their image bytes in hand. The row stays unseen
    // until every card reports in, or the wait runs out, then deals, as on
    // zwipe.net's shared deck.
    let mut featured_loaded = use_signal(|| 0usize);
    let mut featured_timed_out = use_signal(|| false);
    use_future(move || async move {
        timer::sleep(FEATURED_WAIT).await;
        featured_timed_out.set(true);
    });
    let featured_ready = featured_timed_out() || featured_loaded() >= featured.len();
    let featured_class = if featured_ready {
        "deck-head-deal ready"
    } else {
        "deck-head-deal"
    };
    let on_featured_load = move |()| {
        let loaded = *featured_loaded.peek();
        featured_loaded.set(loaded.saturating_add(1));
    };
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
    let mut sections: Vec<(DeckSection, Vec<RowInfo>)> = Vec::new();
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
        ));
    }
    let empty = loaded() && shown_boards.main && sections.is_empty();

    // The fold-all button: every band shut, or every one open again.
    let section_keys: Vec<String> = sections.iter().map(|(s, _)| s.key.clone()).collect();
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

    // The owner's facts, stats and warnings follow the cards as edited here;
    // the store links stay the deck's as fetched.
    let held_deck = deck();
    let metrics = held_deck.as_ref().and_then(|d| {
        zone.read()
            .live_metrics(&d.entries, &entries.read(), &d.command_zone_cards)
    });
    let warnings = held_deck
        .as_ref()
        .map(|d| {
            zone.read()
                .live_warnings(&d.deck_profile, &d.entries, &entries.read())
        })
        .unwrap_or_default();
    let deck_buy = held_deck.as_ref().map(buy::deck_links);
    let has_cards = metrics.is_some();
    let shown = currency();
    let price = metrics.as_ref().and_then(|m| match shown {
        PriceCurrency::Eur => m.total_price_eur,
        PriceCurrency::Tix => m.total_price_tix,
        PriceCurrency::Usd => m.total_price_usd,
    });
    let name = held_profile.as_ref().map(|p| p.name.to_string());
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
        div {
            class: "content-enter web-page web-deck-page",
            onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
            DeckPage {
                head: rsx! {
                if let Some(facts) = facts {
                    DeckHead {
                        name,
                        facts,
                        has_featured: !featured.is_empty(),
                        featured_class: featured_class.to_string(),
                        for (card, role) in featured.iter().cloned() {
                            FeaturedCard {
                                key: "{card.scryfall_data.id}-{role}",
                                card,
                                role: role.to_string(),
                                on_open: move |(sd, shown_face)| preview.set(Some((sd, shown_face))),
                                on_load: on_featured_load,
                            }
                        }
                    }
                } else {
                    DeckHeadGhost {}
                }
            },
            controls: rsx! {
                if !loaded() {
                    DeckControlsGhost {}
                } else {
                    DeckControls {
                        group_by,
                        show_tokens: tokens().is_some_and(|t| !t.is_empty()).then_some(show_tokens),
                        show_zone: zone.read().offers_chip(format).then_some(show_zone),
                        show_art,
                        art_disabled: view() == CardsView::Stacks,
                        all_folded,
                        on_fold_all: fold_all,
                        on_change: move |()| selected.set(None),
                        leading: rsx! {
                            QuickAdd { deck_id, entries, on_added: move |()| {} }
                        },
                        boards: rsx! {
                            // Board chips ease the cards in and out (`settle`).
                            div { class: "chip-row", "data-settle": "",
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
                            CardsViewSwitch { view, on_change: move |()| selected.set(None) }
                        },
                    }
                }
            },
            empty,
            bands: rsx! {
                if !loaded() {
                    DeckBandsGhost {}
                }
                if view() == CardsView::Stacks {
                    {stacks_view(sections, cur, collapsed, stack_details_state, ed.faces, group_buy, dealing())}
                } else {
                    for (section, rows) in sections {
                        DeckBand {
                            key: "{section.key}",
                            accessory: (!group_cards(&rows).is_empty()).then(|| {
                                let header = section.title.clone();
                                let for_buy = rows.clone();
                                rsx! {
                                    CartButton { onclick: move |_| group_buy.offer(&header, &for_buy) }
                                }
                            }),
                            section,
                            folded: collapsed,
                            for row in rows {
                                {row_view(&ed, cur, &row_meta, row)}
                            }
                        }
                    }
                }
            },
            stats: rsx! {
                if !loaded() {
                    DeckStatsGhost {}
                }
                if let Some(p) = held_profile {
                    if has_budget(&p, has_cards) {
                        DeckFold {
                            fold_key: "Budget",
                            title: "Budget",
                            folded,
                            accessory: has_cards.then(|| rsx! { CurrencyChips { currency } }),
                            BudgetBody { profile: p.clone(), metrics, currency }
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
                if !loaded() {
                    DeckFootGhost {}
                } else {
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
                }
            },
            }
        }
        // The page's actions as one floating icon bar at the bottom right:
        // add and remove, then the filter (with its reset while a filter is
        // on) and undo, then the deck's own: edit, clone and share.
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
                "data-action": components::data_action::UNDO,
                disabled: !undo_ready,
                onclick: move |_| apply_undo.call(()),
                DockIcon { path: UNDO }
            }
            span { class: "action-dock-divider" }
            button {
                class: "action-dock-btn",
                "aria-label": EDIT,
                onclick: move |_| {
                    navigator.push(Route::EditDeck { deck_id });
                },
                EditIcon {}
            }
            button {
                class: "action-dock-btn",
                "aria-label": MoreItem::CloneDeck.short_label(),
                onclick: move |_| clone.set(true),
                CloneIcon {}
            }
            button {
                class: "action-dock-btn",
                "aria-label": MoreItem::ShareDeck.short_label(),
                onclick: move |_| share.set(true),
                ShareIcon {}
            }
            if deck_buy.is_some() {
                button {
                    class: "action-dock-btn",
                    "aria-label": buy::CART_LABEL,
                    onclick: move |_| deck_buy_open.set(true),
                    DockIcon { path: CART }
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
        HoverStackHost { stack: hover_stack }
        if let Some(links) = deck_buy {
            StoreDialog {
                open: deck_buy_open,
                title: buy::DECK_TITLE.to_owned(),
                body: buy::DECK_BODY.to_owned(),
                links,
            }
        }
        {group_buy_dialog(group_buy)}
        if view() == CardsView::Stacks {
            {stack_details(&ed, &all_rows, &row_meta, stack_details_state, overlays)}
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
