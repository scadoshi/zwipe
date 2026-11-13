//! The deck on a wide window: the deck page both sites share, editing. The
//! head names the deck over its facts and featured cards, as the shared page
//! does (the page's `?` sits in the nav); the controls
//! add Quick add and the Boards row; the bands are the editor's rows,
//! opening inline with their actions as on the phone, the hover preview
//! pinned to the side the cursor is not on; the stats fold under the cards;
//! and the foot holds Warnings, Data and Danger zone. The dock floats add,
//! remove, the filter and undo, then Edit, Clone and Share.

use components::icons::{CloneIcon, DockIcon, EditIcon, MINUS, PLUS, ShareIcon, UNDO};
use components::{DeckFacts, DeckRowHost, use_stack_details};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Deck, deck_tag_label};
use zwipe_ui::deck::profile::DeckInfo;
use zwipe_ui::deck::view::StatsProfile;
use zwipe_ui::deck_cards::stacks::Reveal;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::screens::deck_view::{EDIT, MoreItem};
use zwipe_ui::{buy, deck_cards};

use super::body::DeckPageBody;
use super::dialogs::DeckDialogs;
use super::panels::{DangerZonePanel, DataPanel};
use super::profile::{DeckFoot, DeckProfilePhone};
use super::sections::{WarningActions, WarningFixes};
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::use_card_overlays;
use crate::inbound::cards::otag::use_otag_reveals;
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::stack_view::{use_cards_view, use_stack_details_sync};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::cards::editor::{Editor, use_deck_editor, use_deck_row_host};
use crate::inbound::screens::deck::cards::quick_add::QuickAdd;

/// The page for `deck_id`. `fixes` are the warnings' fixes, and `fixed`
/// counts the ones that have landed: each reloads the deck. `deck_copy` holds
/// the deck as the page's editor last fetched it, for the host's fixes.
#[component]
pub fn DeckWidePage(
    deck_id: Uuid,
    currency: Signal<PriceCurrency>,
    fixes: WarningFixes,
    fixed: ReadSignal<u32>,
    mut deck_copy: Signal<Option<Deck>>,
    dialogs: DeckDialogs,
    // On a phone, whether the Cards layer is open, what opens and closes it, and the Profile's
    // dock before the cart and Cards.
    cards_open: bool,
    on_cards: Callback<bool>,
    profile_dock: Element,
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
    let stack_details = use_stack_details(reveal, details_open);
    let stack_details_state = use_context_provider(|| stack_details);
    let ed = use_deck_editor(deck_id);
    let Editor {
        deck,
        entries,
        zone,
        profile,
        loaded,
        tokens,
        filter,
        group_by,
        boards,
        show_tokens,
        show_zone,
        show_art,
        collapsed,
        selected,
        printing,
        printing_open,
        filter_open,
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
    // List or Stacks, as last left.
    let view = use_cards_view();
    let listed_ids = use_memo(move || {
        listing()
            .rows
            .iter()
            .map(|r| r.card.scryfall_data.id)
            .collect::<Vec<_>>()
    });
    use_stack_details_sync(reveal, details_open, listed_ids);
    // The host's copy of the deck follows the editor's, the one fetch of it the page makes.
    use_effect(move || {
        let fetched = deck.read().clone();
        if *deck_copy.peek() != fetched {
            deck_copy.set(fetched);
        }
    });
    // A fix made at the foot reloads the deck once it has landed, so the
    // rows, the counts and the warnings follow.
    use_effect(move || {
        if fixed() > 0 {
            ed.reload.call(());
        }
    });

    // A warning's quantity fix goes through the editor, the one deck the Profile and the Cards
    // read, so it records its undo from the copies as they stand, edits in the Cards included.
    let edit = ed.edit_actions();
    let fixes = WarningFixes {
        fix_quantity: Callback::new(move |(card, quantity): (Uuid, i32)| {
            let now = meta.peek().get(&card).map_or(1, |m| m.qty);
            let step = quantity.saturating_sub(now);
            if step != 0 {
                edit.change_quantity.call((card, step, true));
            }
        }),
        ..fixes
    };
    let held_profile = profile();
    let format = held_profile.as_ref().and_then(|p| p.format);
    let oathbreaker = format.is_some_and(|f| f.has_signature_spell());
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

    // The owner's facts, stats and warnings follow the cards as edited here;
    // the store links stay the deck's as fetched.
    let held_deck = deck();
    let metrics = held_deck.as_ref().and_then(|d| {
        zone.read()
            .live_metrics(&d.entries, &entries.read(), &d.command_zone_cards)
    });
    let warnings = held_deck.as_ref().map_or_default(|d| {
        zone.read()
            .live_warnings(&d.deck_profile, &d.entries, &entries.read())
    });
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
    let land_target = held_profile.as_ref().and_then(|p| p.land_target);
    let stats = held_profile.as_ref().map(StatsProfile::from);
    // On a phone, the Profile, read from the cards as edited here, as the wide page's facts are,
    // so an edit in the Cards layer shows on it at once.
    let info = held_profile.as_ref().map(|p| {
        let cards = metrics.as_ref().map_or_else(
            || usize::try_from(p.card_count).unwrap_or(0),
            |m| m.total_cards,
        );
        let commander = zone
            .read()
            .commander
            .as_ref()
            .map(|c| c.scryfall_data.name.clone());
        DeckInfo::of_profile(p, cards, commander)
    });
    let phone_profile = rsx! {
        DeckProfilePhone {
            info,
            stats: stats.clone().unwrap_or_default(),
            metrics: metrics.clone(),
            metrics_loading: !loaded(),
            currency,
            warnings: warnings.clone(),
            actions: Some(WarningActions { deck_id, fixes }),
            data: rsx! {
                DataPanel {
                    import: Some(Route::ImportDeck { deck_id }),
                    export: Route::ExportDeck { deck_id },
                    has_cards,
                }
            },
            danger: rsx! {
                DangerZonePanel { dialogs }
            },
        }
    };

    rsx! {
        DeckPageBody {
            loaded: loaded(),
            name,
            facts,
            featured,
            format,
            listing: listing(),
            meta: meta(),
            group_by,
            show_tokens: tokens().is_some_and(|t| !t.is_empty()).then_some(show_tokens),
            show_zone: zone.read().offers_chip(format).then_some(show_zone),
            show_art,
            boards,
            has_maybe,
            has_side,
            view,
            collapsed,
            selected,
            face: ed.face,
            faces: ed.faces,
            details: stack_details_state,
            filter,
            filter_open,
            on_reset_filter: reset_filter,
            sources,
            row_currency: cur,
            currency,
            stats,
            metrics,
            land_target,
            deck_buy,
            actions: Some(ed.edit_actions()),
            host: row_host,
            drawer_host: DeckRowHost { on_view_printing: None, ..row_host },
            overlays,
            examples,
            cards_open,
            on_cards,
            // The nav's `?` over the Cards opens their hint.
            cards_hint: Callback::new(move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            }),
            phone_profile,
            profile_dock,
            // The Cards layer's undo, after the filter.
            cards_dock_tail: rsx! {
                button {
                    class: "action-dock-btn",
                    "aria-label": "Undo",
                    "data-action": components::data_action::UNDO,
                    "data-settle": "",
                    disabled: !undo_ready,
                    onclick: move |_| apply_undo.call(()),
                    DockIcon { path: UNDO }
                }
            },
            quick_add: rsx! {
                QuickAdd { deck_id, entries, on_added: move |()| {} }
            },
            foot: rsx! {
                DeckFoot {
                    loaded: loaded(),
                    warnings,
                    actions: Some(WarningActions { deck_id, fixes }),
                    data: rsx! {
                        DataPanel {
                            import: Some(Route::ImportDeck { deck_id }),
                            export: Route::ExportDeck { deck_id },
                            has_cards,
                        }
                    },
                    danger: rsx! {
                        DangerZonePanel { dialogs }
                    },
                }
            },
            // Add and remove, before the filter.
            dock_lead: rsx! {
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
            },
            // Undo after the filter, then the deck's own: edit, clone and share.
            dock_tail: rsx! {
                button {
                    class: "action-dock-btn",
                    "aria-label": "Undo",
                    "data-action": components::data_action::UNDO,
                    // Undo puts a card back, so the page settles around it.
                    "data-settle": "",
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
            },
            extra: rsx! {
                OneTimeHintDialog { hint: OneTimeHint::DeckCards, open: hint_open }
                if let Some((card, _)) = printing() {
                    PrintingSheet {
                        key: "{card.scryfall_data.id}",
                        card,
                        open: printing_open,
                        on_save: move |picked: Card| save_printing.call(picked),
                    }
                }
            },
        }
    }
}
