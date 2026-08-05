use super::components::{
    card_row::{CardRow, OtagDescribe, OtagExamplesOpen, ShowRowArt},
    filter_store::{FilterScope, FilterStore},
    image_preview::ImagePreview,
    printing_sheet::PrintingSheet,
    quick_add::QuickAdd,
    undo_log::{CommandZoneSlot, UndoAction, UndoLog, UndoStore},
};
use crate::{
    inbound::{
        components::{
            auth::ensure_session::EnsureFresh,
            catalog_cache::CatalogCache,
            chip::Chip,
            hint_dialog::{
                HintBullet, HintBullets, HintColored, HintDialog, HintKey, HintLine,
                open_and_record_hint,
            },
            screen_header::ScreenHeader,
            telemetry::{
                usage_buffer::UsageBuffer,
                vocabulary::{component, screen},
            },
        },
        screens::{
            deck::{
                card::filter::{
                    card_filter_sheet::{CardFilterSheet, CollapseExpanded},
                    deck_cards::DeckCards,
                },
                components::{featured_cards::FeaturedCards, skeletons::DeckCardListSkeleton},
            },
            oracle_tag_examples::OracleTagExamples,
        },
    },
    outbound::client::{
        ClientError, ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{
            get_deck::ClientGetDeck, get_deck_profile::ClientGetDeckProfile,
            get_deck_tokens::ClientGetDeckTokens, update_deck_profile::ClientUpdateDeckProfile,
        },
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
            update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::{core::spawn_forever, prelude::*};
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;
use uuid::Uuid;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{
            Card,
            search_card::{
                card_filter::{
                    builder::CardQueryBuilder, card_sort_key::CardSortKey,
                    price_currency::PriceCurrency,
                },
                cards::{Cards, sort_deck_entries},
                group_cards::{CardGroup, GroupByOption, GroupCards},
            },
        },
        deck::{
            Board, DeckEntry, DeckOtherTag, Format, PowerLevel,
            deck_metrics::{budget_tier, deck_price},
            deck_tag_label,
            quantity::Quantity,
        },
        user::models::hints::HINT_DECK_CARDS,
    },
    http::{
        contracts::{
            deck::HttpUpdateDeckProfile,
            deck_card::{HttpCreateDeckCard, HttpPatchDeckCard},
        },
        helpers::Opdate,
    },
};

/// Per-card render metadata (quantity, board, MVP flag) indexed by printing id
/// (`scryfall_data.id`). Built once per `deck_entries` change so the card-row
/// render does O(1) lookups instead of scanning the whole entry list per card
/// (the list can hold up to `MAX_CARDS_PER_DECK` entries).
#[derive(Clone, Copy, PartialEq)]
struct DeckRowMeta {
    qty: i32,
    board: Board,
    mvp: bool,
}

/// A card's accumulated, not-yet-posted quantity delta. Taps update the UI
/// immediately; the server gets one net call per card once taps stop for
/// [`QTY_FLUSH_MS`] (or on screen exit). One in-flight call per burst also
/// gives rollback a single stable baseline, where the previous per-tap calls
/// could overlap and race their rollbacks.
#[derive(Clone, Copy)]
struct PendingQty {
    /// Quantity before the burst — the delete threshold and rollback target.
    baseline: i32,
    /// Net delta across the burst.
    delta: i32,
    /// Bumped per tap; a waking flush task only posts if its generation is
    /// still the latest.
    generation: u32,
}

/// Quiet period after the last quantity tap before the net delta posts.
const QTY_FLUSH_MS: u64 = 300;

#[component]
pub fn View(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    // This screen's own filter: seeded from the per-(screen, deck) store,
    // provided as context so the filter modules bind to it, parked on leave.
    let filter_store: FilterStore = use_context();
    let mut filter_builder: Signal<CardQueryBuilder> = use_signal(|| {
        filter_store
            .restore(FilterScope::Cards, deck_id)
            .unwrap_or_default()
    });
    use_context_provider(|| filter_builder);
    use_drop(move || {
        let mut store = filter_store;
        store.park(FilterScope::Cards, deck_id, filter_builder.peek().clone());
    });

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    // Incrementing this re-runs the filter + group effect
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();

    // Deck identity for the header block (name + format/power/tag chips, like
    // the zite share page).
    let mut deck_name: Signal<String> = use_signal(String::new);
    let mut deck_format: Signal<Option<Format>> = use_signal(|| None);
    let mut deck_power: Signal<Option<PowerLevel>> = use_signal(|| None);
    let mut deck_tags: Signal<Vec<String>> = use_signal(Vec::new);
    let mut deck_other_tags: Signal<Vec<DeckOtherTag>> = use_signal(Vec::new);
    // Source of truth — all non-commander entries (active + maybeboard)
    let mut deck_entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);
    // Command-zone cards (commander, partner, etc.), folded into the budget total.
    let mut command_zone_cards: Signal<Vec<Card>> = use_signal(Vec::new);

    // Undo log — mutations push their inverses; the ActionBar's conditional
    // Undo button pops them (apply_undo below). Provided as context so the
    // quick-add bar can record its adds. Seeded from the app-level per-deck
    // store on mount and parked back on leave, so the stack survives
    // navigating around the app (like the filter above).
    let undo_store: UndoStore = use_context();
    let undo_log = UndoLog(use_signal(|| undo_store.restore(deck_id)));
    use_context_provider(|| undo_log);
    use_drop(move || {
        let mut store = undo_store;
        store.park(deck_id, undo_log.0.peek().clone());
    });

    // Per-card pending quantity bursts (see PendingQty).
    let pending_qty: Signal<HashMap<Uuid, PendingQty>> = use_signal(HashMap::new);
    // A tap inside the debounce window must still reach the server when the
    // user leaves the screen: post every mapped burst on unmount. Detached
    // tasks (the screen's scope is going away), so failures only report and
    // toast — no local state to roll back.
    use_drop(move || {
        let bursts: Vec<(Uuid, PendingQty)> = pending_qty
            .peek()
            .iter()
            .map(|(card_id, p)| (*card_id, *p))
            .collect();
        for (card_id, p) in bursts {
            if p.delta == 0 {
                continue;
            }
            spawn_forever(async move {
                let Ok(session) = session.ensure_fresh(client).await else {
                    return;
                };
                let result = if p.baseline + p.delta < 1 {
                    client()
                        .delete_deck_card(deck_id, card_id, &session)
                        .await
                        .map(|_| ())
                } else {
                    // PATCH: the burst's absolute target, replay-safe.
                    let request = HttpPatchDeckCard::new(Some(p.baseline + p.delta), None);
                    client()
                        .update_deck_card(deck_id, card_id, &request, &session)
                        .await
                        .map(|_| ())
                };
                if let Err(e) = result {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::NONE,
                        "flush_quantity",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            });
        }
    });

    // Provide Card list context for filter sheet (derives from all entries)
    let mut deck_cards_for_filter: Signal<Vec<Card>> = use_signal(Vec::new);
    use_context_provider(|| DeckCards(deck_cards_for_filter));

    // Guards filter effect from running before the deck has loaded
    let mut deck_loaded: Signal<bool> = use_signal(|| false);

    // Effective land target (deck override, else format heuristic) for the
    // land-count crossing toasts on quantity changes.
    let mut land_target: Signal<Option<i32>> = use_signal(|| None);
    // Deck price budget (None = no budget) + its currency, for the 50/75/100%
    // crossing toasts on quantity changes.
    let mut price_budget: Signal<Option<f64>> = use_signal(|| None);
    let mut price_budget_currency: Signal<PriceCurrency> = use_signal(|| PriceCurrency::Usd);
    // Shared with every CardRow so compact-row prices use the deck's currency.
    use_context_provider(|| price_budget_currency);

    // Oracle-tag reveal inside expanded rows: a description lookup (from the shared
    // catalog cache) and an examples-browse opener, handed to every CardRow via
    // context. The examples overlay is owned here (this screen has no transform, so
    // its fixed overlay resolves to the viewport, unlike the filter sheet).
    let cache: CatalogCache = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });
    let describe_tag = use_callback(move |slug: String| {
        cache.oracle_tags.cell().read().loaded().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    });
    let mut otag_examples_open = use_signal(|| false);
    let mut otag_examples_slug = use_signal(String::new);
    let open_examples = use_callback(move |slug: String| {
        otag_examples_slug.set(slug);
        otag_examples_open.set(true);
    });
    use_context_provider(|| OtagDescribe(describe_tag));
    use_context_provider(|| OtagExamplesOpen(open_examples));
    // Art-crop thumbnails on the card rows, on by default; the "Art" chip in
    // the controls toggles it. Screen-local for now (a synced preference can
    // come later if people want it sticky).
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    // What the UI renders — grouped card lists (active cards only)
    let mut displayed_groups: Signal<Vec<CardGroup>> = use_signal(Vec::new);
    // Active lands, pulled out of the group-by pipeline and pinned to the bottom
    // in their own section (filtered + sorted like the groups above).
    let mut displayed_lands: Signal<Vec<Card>> = use_signal(Vec::new);
    // Current grouping mode
    let mut group_by_option: Signal<GroupByOption> = use_signal(|| GroupByOption::CardType);
    // Which card row is expanded (None = all collapsed)
    let mut expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    // Commander is always pinned — never part of groupable entries
    let mut commander_card: Signal<Option<Card>> = use_signal(|| None);
    // Signature spell pinned alongside commander (Oathbreaker only)
    let mut signature_spell_card: Signal<Option<Card>> = use_signal(|| None);
    // Whether the format is Oathbreaker (for label swapping)
    let mut is_oathbreaker: Signal<bool> = use_signal(|| false);
    // Commander filtered by the active filter (None = filtered out or no commander)
    let mut displayed_commander: Signal<Option<Card>> = use_signal(|| None);
    // Signature spell filtered by the active filter
    let mut displayed_signature_spell: Signal<Option<Card>> = use_signal(|| None);
    // Partner commander pinned alongside commander
    let mut partner_card: Signal<Option<Card>> = use_signal(|| None);
    let mut displayed_partner: Signal<Option<Card>> = use_signal(|| None);
    // Background enchantment pinned in its own group
    let mut background_card: Signal<Option<Card>> = use_signal(|| None);
    let mut displayed_background: Signal<Option<Card>> = use_signal(|| None);
    // Toggle to show/hide tokens at the top of the list
    let mut show_tokens: Signal<bool> = use_signal(|| false);
    // Toggle to show/hide command zone pinned sections (default: shown)
    let mut show_command_zone: Signal<bool> = use_signal(|| true);
    // Board filter — multi-select toggles (deck is always on)
    let mut show_deck: Signal<bool> = use_signal(|| true);
    let mut show_maybe: Signal<bool> = use_signal(|| false);
    let mut show_side: Signal<bool> = use_signal(|| false);

    // Card image preview — stores the card to preview (None = closed). Carries the
    // full ScryfallData so the modal's FlippableCardImage can read both faces of DFCs.
    let preview_card: Signal<
        Option<(zwipe_core::domain::card::scryfall_data::ScryfallData, usize)>,
    > = use_signal(|| None);
    // Printing sheet state
    let mut printing_sheet_open: Signal<bool> = use_signal(|| false);
    let mut printing_sheet_card: Signal<Option<Card>> = use_signal(|| None);
    let mut command_zone_slot: Signal<Option<CommandZoneSlot>> = use_signal(|| None);
    // Controls the dismiss animation before clearing the URL
    let preview_dismissing: Signal<bool> = use_signal(|| false);
    // Whether the next filter effect run should collapse the expanded card
    // (true for structural changes like group-by/filter/board; false for qty bumps).
    // Provided to the filter sheet via the `CollapseExpanded` newtype so the
    // context lookup is unambiguous.
    let mut should_collapse_expanded: Signal<bool> = use_signal(|| false);
    use_context_provider(|| CollapseExpanded(should_collapse_expanded));

    // Deck-browsing hint: fires once entries have loaded and only if the
    // deck actually has cards. An empty list has nothing to tap, so the
    // hint waits for a later visit instead of burning its one showing.
    let deck_cards_hint_open = use_signal(|| false);
    let mut deck_cards_hint_fired = use_signal(|| false);
    use_effect(move || {
        if !deck_entries.read().is_empty() && !*deck_cards_hint_fired.peek() {
            deck_cards_hint_fired.set(true);
            open_and_record_hint(HINT_DECK_CARDS, session, client, deck_cards_hint_open);
        }
    });

    // Helper: look up quantity by card ID
    let qty_for = move |card_id: Uuid| -> i32 {
        deck_entries
            .peek()
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
            .map(|e| *e.deck_card.quantity)
            .unwrap_or(1)
    };

    // Index deck entries by printing id so the card-row render resolves
    // qty/board/mvp in O(1) instead of scanning `deck_entries` per card
    // (previously ~3 linear scans per row → O(n^2) across the list). First
    // occurrence wins, matching the prior `.find()` semantics. Recomputes only
    // when `deck_entries` changes.
    let row_meta: Memo<HashMap<Uuid, DeckRowMeta>> = use_memo(move || {
        let entries = deck_entries.read();
        let mut map = HashMap::with_capacity(entries.len());
        for e in entries.iter() {
            map.entry(e.card.scryfall_data.id)
                .or_insert_with(|| DeckRowMeta {
                    qty: *e.deck_card.quantity,
                    board: e.deck_card.board,
                    mvp: e.deck_card.mvp_at.is_some(),
                });
        }
        map
    });

    // Effect 1 — mount load (reads `session` reactively)
    // Fetches deck entries, separates the commander into its own pinned slot.
    use_effect(move || {
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(_) => {
                    return;
                }
            };

            let mut entries = match client().get_deck(deck_id, &session).await {
                Ok(deck) => {
                    command_zone_cards.set(deck.command_zone_cards);
                    deck.entries
                }
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::NONE,
                        "load_deck",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };

            // Resolve commander and signature spell from profile.
            // Pull from entries if present, otherwise fetch separately.
            if let Ok(profile) = client().get_deck_profile(deck_id, &session).await {
                deck_name.set(profile.name.to_string());
                deck_format.set(profile.format);
                deck_power.set(profile.power_level);
                deck_tags.set(profile.tags.clone());
                deck_other_tags.set(profile.other_tags.clone());
                is_oathbreaker.set(
                    profile
                        .format
                        .as_ref()
                        .is_some_and(|f| f.has_signature_spell()),
                );
                // Explicit target only — no toasts unless the user set one.
                land_target.set(profile.land_target);
                price_budget.set(profile.price_target);
                price_budget_currency
                    .set(profile.price_target_currency.unwrap_or(PriceCurrency::Usd));

                // Resolve command zone cards by oracle_id (not printing-specific scryfall_data_id).
                // Fetch the card first to get its oracle_id, then remove from entries by oracle_id.
                if let Some(commander_id) = profile.commander_id {
                    let fetched = client().get_card(commander_id).await.ok();
                    if let Some(ref card) = fetched
                        && let Some(oid) = card.scryfall_data.oracle_id
                    {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    commander_card.set(fetched);
                }

                if let Some(spell_id) = profile.signature_spell_id {
                    let fetched = client().get_card(spell_id).await.ok();
                    if let Some(ref card) = fetched
                        && let Some(oid) = card.scryfall_data.oracle_id
                    {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    signature_spell_card.set(fetched);
                }

                if let Some(partner_id) = profile.partner_commander_id {
                    let fetched = client().get_card(partner_id).await.ok();
                    if let Some(ref card) = fetched
                        && let Some(oid) = card.scryfall_data.oracle_id
                    {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    partner_card.set(fetched);
                }

                if let Some(bg_id) = profile.background_id {
                    let fetched = client().get_card(bg_id).await.ok();
                    if let Some(ref card) = fetched
                        && let Some(oid) = card.scryfall_data.oracle_id
                    {
                        entries.retain(|e| e.card.scryfall_data.oracle_id != Some(oid));
                    }
                    background_card.set(fetched);
                }
            }

            // Update DeckCards context for filter sheet (all cards including maybeboard)
            let all_cards: Vec<Card> = entries.iter().map(|e| e.card.clone()).collect();
            deck_cards_for_filter.set(all_cards);

            deck_entries.set(entries);
            deck_loaded.set(true);

            if !filter_builder.peek().is_empty() {
                toast.warning(
                    "Filter is active".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(2000)),
                );
            }

            let current = *filter_reset_counter.peek();
            filter_reset_counter.set(current + 1);
        });
    });

    let tokens_resource: Resource<Result<Vec<Card>, ClientError>> =
        use_resource(move || async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(_) => {
                    return Ok(Vec::new());
                }
            };
            client().get_deck_tokens(deck_id, &session).await
        });

    // Effect 2 — filter + group (reads `filter_reset_counter`, `group_by_option`, `board_filter` reactively)
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = group_by_option();

        if !*deck_loaded.peek() {
            return;
        }

        // Only active deck cards go through the group-by pipeline.
        // Maybeboard and sideboard are rendered as their own sections.
        let active_cards: Vec<Card> = deck_entries
            .peek()
            .iter()
            .filter(|e| e.deck_card.board.is_active())
            .map(|e| e.card.clone())
            .collect();

        let builder = filter_builder.peek().clone();
        let group_option = *group_by_option.peek();
        let cmd = commander_card.peek().clone();
        let spell = signature_spell_card.peek().clone();
        let partner = partner_card.peek().clone();
        let bg = background_card.peek().clone();

        // Pinned cards use the bare predicate — a one-card membership test.
        let filter_pinned =
            |card: Option<Card>,
             criteria: &zwipe_core::domain::card::search_card::card_filter::CardCriteria|
             -> Option<Card> { card.filter(|c| criteria.matches(c)) };

        let (filtered, new_cmd, new_spell, new_partner, new_bg) = if builder.is_empty() {
            (active_cards, cmd, spell, partner, bg)
        } else {
            match builder.build_criteria() {
                Ok(criteria) => {
                    let filtered: Vec<Card> = Cards::from(active_cards).matching(&criteria).into();
                    (
                        filtered,
                        filter_pinned(cmd, &criteria),
                        filter_pinned(spell, &criteria),
                        filter_pinned(partner, &criteria),
                        filter_pinned(bg, &criteria),
                    )
                }
                Err(_) => (active_cards, cmd, spell, partner, bg),
            }
        };

        // No sort chosen -> alphabetical, so the deck list always has a stable,
        // scannable order (applies to tokens/maybeboard/sideboard below too).
        let filtered: Vec<Card> = Cards::from(filtered)
            .sorted(
                builder.sort().unwrap_or(CardSortKey::Name),
                builder.ascending(),
            )
            .into();

        // Lands render in their own pinned section (below), out of the group-by
        // pipeline; only the nonland cards are grouped. Both stay filtered + sorted.
        let (lands, nonlands): (Vec<Card>, Vec<Card>) = filtered
            .into_iter()
            .partition(|c| c.scryfall_data.is_land());
        let groups = nonlands.group_by(group_option);
        displayed_groups.set(groups);
        displayed_lands.set(lands);
        displayed_commander.set(new_cmd);
        displayed_signature_spell.set(new_spell);
        displayed_partner.set(new_partner);
        displayed_background.set(new_bg);
        if *should_collapse_expanded.peek() {
            expanded_card.set(None);
            should_collapse_expanded.set(false);
        }
    });

    // Mainboard land count from the source of truth — quantity-aware (10 land
    // rows at ×4 = 40), MDFC land faces included via is_land. Recomputed each
    // call so it never drifts.
    let main_land_count = move || -> i32 {
        deck_entries
            .peek()
            .iter()
            .filter(|e| e.deck_card.board.is_active() && e.card.scryfall_data.is_land())
            .map(|e| *e.deck_card.quantity)
            .sum()
    };

    // Toast when a land change crosses the deck's land target in either
    // direction: below → warning, back to/above → reached.
    let warn_land_crossing = move |before: i32, after: i32| {
        if let Some(target) = land_target() {
            if before >= target && after < target {
                toast.warning(
                    format!("Below land target ({target})"),
                    ToastOptions::default().duration(Duration::from_millis(2500)),
                );
            } else if before < target && after >= target {
                toast.info(
                    format!("Land target reached ({target})"),
                    ToastOptions::default().duration(Duration::from_millis(2500)),
                );
            }
        }
    };

    // Mainboard total in the budget currency from the source of truth.
    let total_price = move || -> f64 {
        deck_price(
            &deck_entries.peek(),
            &command_zone_cards.peek(),
            price_budget_currency(),
        )
    };

    // Toast once when a qty change raises the deck into a new budget tier
    // (50/75/100%). Higher-tier only, so it never re-fires.
    let warn_budget_crossing = move |before: f64, after: f64| {
        let Some(budget) = price_budget() else {
            return;
        };
        if budget <= 0.0 {
            return;
        }
        if budget_tier(budget, after) > budget_tier(budget, before) {
            let pct = after / budget * 100.0;
            let amount = price_budget_currency().format_amount(budget);
            let msg = format!("Deck at {pct:.2}% of your {amount} budget");
            let opts = ToastOptions::default().duration(Duration::from_millis(2500));
            if after >= budget {
                toast.warning(msg, opts);
            } else {
                toast.info(msg, opts);
            }
        }
    };

    // `record: false` marks an undo-driven call — it must not push a fresh
    // undo entry (no ping-pong) and skips the user-action toasts its inverse
    // already covers.
    let mut change_quantity =
        move |card_id: Uuid, delta: i32, _is_basic_land: bool, record: bool| {
            let current_qty = qty_for(card_id);

            // - at 1 → delete
            let should_delete = current_qty + delta < 1;
            let before_lands = main_land_count();
            let before_price = total_price();

            // Optimistic UI per tap; the server call is debounced below.
            if should_delete {
                let removed_entry = deck_entries
                    .peek()
                    .iter()
                    .find(|e| e.card.scryfall_data.id == card_id)
                    .cloned();

                // Suggestion signal: a deliberate removal (delayed negative), keyed
                // by the deck's commander + the removed card's oracle id.
                let card_oracle_id = removed_entry
                    .as_ref()
                    .and_then(|e| e.card.scryfall_data.oracle_id);
                usage_buffer().record_removal(deck_id, card_oracle_id);

                // Undoable: re-creating wants the old board + the quantity from
                // before this whole burst, not the 1 it died at.
                if record && let Some(entry) = removed_entry {
                    let baseline = pending_qty
                        .peek()
                        .get(&card_id)
                        .map_or(current_qty, |p| p.baseline);
                    undo_log.push(UndoAction::Removed { entry, baseline });
                }

                // Optimistic: remove from entries
                deck_entries
                    .write()
                    .retain(|e| e.card.scryfall_data.id != card_id);
                // Trigger re-filter
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);

                warn_land_crossing(before_lands, main_land_count());
                warn_budget_crossing(before_price, total_price());

                toast.info(
                    "Card removed".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            } else {
                // Optimistic: update quantity in entries
                if let Some(entry) = deck_entries
                    .write()
                    .iter_mut()
                    .find(|e| e.card.scryfall_data.id == card_id)
                    && let Ok(new_qty) = Quantity::new(current_qty + delta)
                {
                    entry.deck_card.quantity = new_qty;
                }
                // Trigger re-render so qty display updates
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);

                warn_land_crossing(before_lands, main_land_count());
                warn_budget_crossing(before_price, total_price());
            }

            // Fold the tap into the card's pending burst and (re)arm its debounce.
            let generation = {
                let mut pending_qty = pending_qty;
                let mut map = pending_qty.write();
                let entry = map.entry(card_id).or_insert(PendingQty {
                    baseline: current_qty,
                    delta: 0,
                    generation: 0,
                });
                entry.delta += delta;
                entry.generation = entry.generation.wrapping_add(1);
                entry.generation
            };

            spawn(async move {
                sleep(Duration::from_millis(QTY_FLUSH_MS)).await;

                // Only the burst's latest tap posts; earlier taps wake to a newer
                // generation and stand down. Navigating away cancels these tasks
                // instead — the use_drop exit flush posts whatever is still mapped.
                let flushed = {
                    let mut pending_qty = pending_qty;
                    let mut map = pending_qty.write();
                    match map.get(&card_id) {
                        Some(p) if p.generation == generation => map.remove(&card_id),
                        _ => None,
                    }
                };
                let Some(pending) = flushed else {
                    return;
                };
                if pending.delta == 0 {
                    // A burst that nets out (+1 then -1) needs no server call.
                    return;
                }

                let session = match session.ensure_fresh(client).await {
                    Ok(session) => session,
                    Err(e) => {
                        usage_buffer.peek().report_error(
                            screen::DECK_CARD_VIEW,
                            component::NONE,
                            "change_quantity",
                            &e,
                        );
                        toast.error(e.to_user_message(), ToastOptions::default());
                        return;
                    }
                };

                if pending.baseline + pending.delta < 1 {
                    if let Err(e) = client().delete_deck_card(deck_id, card_id, &session).await {
                        usage_buffer.peek().report_error(
                            screen::DECK_CARD_VIEW,
                            component::NONE,
                            "change_quantity",
                            &e,
                        );
                        toast.error(e.to_user_message(), ToastOptions::default());
                    }
                } else {
                    // PATCH: the burst's absolute target (baseline + net), so
                    // a retried or replayed request lands on the same value.
                    let request =
                        HttpPatchDeckCard::new(Some(pending.baseline + pending.delta), None);
                    match client()
                        .update_deck_card(deck_id, card_id, &request, &session)
                        .await
                    {
                        Ok(_) => {
                            // One burst, one undo entry — recorded only once the
                            // server took it, so undo never inverts a rolled-back
                            // burst.
                            if record
                                && let Some(entry) = deck_entries
                                    .peek()
                                    .iter()
                                    .find(|e| e.card.scryfall_data.id == card_id)
                            {
                                undo_log.push(UndoAction::QuantityChanged {
                                    card_id,
                                    card_name: entry.card.scryfall_data.name.clone(),
                                    net: pending.delta,
                                    baseline: pending.baseline,
                                });
                            }
                        }
                        Err(e) => {
                            usage_buffer.peek().report_error(
                                screen::DECK_CARD_VIEW,
                                component::NONE,
                                "change_quantity",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                            // Roll the whole burst back to its pre-burst quantity.
                            if let Some(entry) = deck_entries
                                .write()
                                .iter_mut()
                                .find(|e| e.card.scryfall_data.id == card_id)
                                && let Ok(reverted) = Quantity::new(pending.baseline)
                            {
                                entry.deck_card.quantity = reverted;
                            }
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                        }
                    }
                }
            });
        };

    let mut move_to_board = move |card_id: Uuid, target: Board, record: bool| {
        // Save old board for rollback (and the name for the undo entry)
        let (old_board, card_name) = deck_entries
            .peek()
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
            .map(|e| (e.deck_card.board, e.card.scryfall_data.name.clone()))
            .unwrap_or((Board::Deck, String::new()));

        if record {
            undo_log.push(UndoAction::MovedBoard {
                card_id,
                card_name,
                from: old_board,
            });
        }

        // Optimistic: set the board
        if let Some(entry) = deck_entries
            .write()
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
        {
            entry.deck_card.board = target;
        }
        let current = *filter_reset_counter.peek();
        filter_reset_counter.set(current + 1);

        let request = HttpPatchDeckCard::new(None, Some(target.display_name().to_string()));
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::NONE,
                        "move_board",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            if let Err(e) = client()
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                usage_buffer.peek().report_error(
                    screen::DECK_CARD_VIEW,
                    component::NONE,
                    "move_board",
                    &e,
                );
                toast.error(e.to_user_message(), ToastOptions::default());
                // Rollback
                if let Some(entry) = deck_entries
                    .write()
                    .iter_mut()
                    .find(|e| e.card.scryfall_data.id == card_id)
                {
                    entry.deck_card.board = old_board;
                }
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
            }
        });

        // Undo emits its own named toast instead.
        if record {
            toast.info(
                format!("Moved to {}", target.display_name()),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
        }
    };

    // Star/unstar a deck MVP (context/plans/deck_mvps/): optimistic flip, the
    // server is the referee for the 3-per-deck cap (its 422 message shows
    // verbatim) and owns the real vesting clock, which the success arm adopts.
    let mut toggle_mvp = move |card_id: Uuid, currently_mvp: bool| {
        let target = !currently_mvp;
        let old_mvp_at = deck_entries
            .peek()
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
            .and_then(|e| e.deck_card.mvp_at);
        if let Some(entry) = deck_entries
            .write()
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
        {
            entry.deck_card.mvp_at = target.then(chrono::Utc::now);
        }
        let current = *filter_reset_counter.peek();
        filter_reset_counter.set(current + 1);

        let request = HttpPatchDeckCard::with_mvp(target);
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::NONE,
                        "toggle_mvp",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            match client()
                .update_deck_card(deck_id, card_id, &request, &session)
                .await
            {
                Ok(updated) => {
                    // Adopt the server's timestamp — the vesting clock.
                    if let Some(entry) = deck_entries
                        .write()
                        .iter_mut()
                        .find(|e| e.card.scryfall_data.id == card_id)
                    {
                        entry.deck_card.mvp_at = updated.mvp_at;
                    }
                }
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_CARD_VIEW,
                        component::NONE,
                        "toggle_mvp",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                    // Rollback
                    if let Some(entry) = deck_entries
                        .write()
                        .iter_mut()
                        .find(|e| e.card.scryfall_data.id == card_id)
                    {
                        entry.deck_card.mvp_at = old_mvp_at;
                    }
                    let current = *filter_reset_counter.peek();
                    filter_reset_counter.set(current + 1);
                }
            }
        });
    };

    // Pop the newest undo entry and apply its inverse. Consumes the entry
    // even when the world has moved on (the inverse no longer applies) — a
    // stale entry gets an "Already changed" toast instead of a doomed
    // request. Undo-driven mutations pass record: false so they never push
    // fresh entries (no ping-pong).
    let mut apply_undo = move || {
        let mut undo_log = undo_log;
        let Some(action) = undo_log.0.write().pop() else {
            return;
        };
        let entry_exists = |id: Uuid| {
            deck_entries
                .peek()
                .iter()
                .any(|e| e.card.scryfall_data.id == id)
        };
        let stale = move || {
            toast.info(
                "Already changed".to_string(),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
        };
        let short = ToastOptions::default().duration(Duration::from_millis(1500));

        match action {
            UndoAction::Added { card_id, card_name } => {
                if !entry_exists(card_id) {
                    stale();
                    return;
                }
                // Not a deliberate removal — no record_removal signal here.
                deck_entries
                    .write()
                    .retain(|e| e.card.scryfall_data.id != card_id);
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
                toast.info(format!("Removed {card_name}"), short);

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            usage_buffer.peek().report_error(
                                screen::DECK_CARD_VIEW,
                                component::NONE,
                                "undo",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                            return;
                        }
                    };
                    if let Err(e) = client().delete_deck_card(deck_id, card_id, &session).await {
                        usage_buffer.peek().report_error(
                            screen::DECK_CARD_VIEW,
                            component::NONE,
                            "undo",
                            &e,
                        );
                        toast.error(e.to_user_message(), ToastOptions::default());
                    }
                });
            }
            UndoAction::Removed { entry, baseline } => {
                let card_id = entry.card.scryfall_data.id;
                if entry_exists(card_id) {
                    // The card found its way back since (re-added manually).
                    stale();
                    return;
                }
                let card_name = entry.card.scryfall_data.name.clone();
                let board = entry.deck_card.board;
                let request = HttpCreateDeckCard::new(
                    &entry.card.scryfall_data,
                    baseline,
                    match board {
                        Board::Deck => None,
                        other => Some(other.display_name().to_string()),
                    },
                );

                // Optimistic re-add at the pre-burst quantity.
                let mut restored = entry;
                if let Ok(qty) = Quantity::new(baseline) {
                    restored.deck_card.quantity = qty;
                }
                deck_entries.write().push(restored);
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
                toast.info(format!("Re-added {card_name}"), short);

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            usage_buffer.peek().report_error(
                                screen::DECK_CARD_VIEW,
                                component::NONE,
                                "undo",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                            return;
                        }
                    };
                    match client().create_deck_card(deck_id, &request, &session).await {
                        Ok(deck_card) => {
                            // Adopt the server row (fresh ids).
                            if let Some(entry) = deck_entries
                                .write()
                                .iter_mut()
                                .find(|e| e.card.scryfall_data.id == card_id)
                            {
                                entry.deck_card = deck_card;
                            }
                        }
                        Err(e) => {
                            usage_buffer.peek().report_error(
                                screen::DECK_CARD_VIEW,
                                component::NONE,
                                "undo",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                            deck_entries
                                .write()
                                .retain(|e| e.card.scryfall_data.id != card_id);
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                        }
                    }
                });
            }
            UndoAction::QuantityChanged {
                card_id,
                card_name,
                net,
                baseline,
            } => {
                if !entry_exists(card_id) {
                    stale();
                    return;
                }
                change_quantity(card_id, -net, false, false);
                toast.info(format!("{card_name} back to \u{00d7}{baseline}"), short);
            }
            UndoAction::MovedBoard {
                card_id,
                card_name,
                from,
            } => {
                if !entry_exists(card_id) {
                    stale();
                    return;
                }
                move_to_board(card_id, from, false);
                toast.info(
                    format!("{card_name} back to {}", from.display_name()),
                    short,
                );
            }
            UndoAction::PrintingChanged { old_card, new_id } => {
                if !entry_exists(new_id) {
                    stale();
                    return;
                }
                let card_name = old_card.scryfall_data.name.clone();
                let old_id = old_card.scryfall_data.id;
                let request = HttpPatchDeckCard::with_printing(&old_id.to_string());
                if let Some(entry) = deck_entries
                    .write()
                    .iter_mut()
                    .find(|e| e.card.scryfall_data.id == new_id)
                {
                    entry.card = old_card;
                    entry.deck_card.scryfall_data_id = old_id;
                }
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
                toast.info(format!("{card_name} printing restored"), short);

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            usage_buffer.peek().report_error(
                                screen::DECK_CARD_VIEW,
                                component::NONE,
                                "undo",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                            return;
                        }
                    };
                    if let Err(e) = client()
                        .update_deck_card(deck_id, new_id, &request, &session)
                        .await
                    {
                        usage_buffer.peek().report_error(
                            screen::DECK_CARD_VIEW,
                            component::NONE,
                            "undo",
                            &e,
                        );
                        toast.error(e.to_user_message(), ToastOptions::default());
                    }
                });
            }
            UndoAction::CommandZonePrintingChanged {
                slot,
                old_card,
                new_id,
            } => {
                // The slot must still hold the printing this entry swapped
                // in — a commander swapped or cleared since makes the
                // inverse stale.
                let mut slot_signal = match slot {
                    CommandZoneSlot::Commander => commander_card,
                    CommandZoneSlot::Partner => partner_card,
                    CommandZoneSlot::Background => background_card,
                    CommandZoneSlot::SignatureSpell => signature_spell_card,
                };
                let holds_new = slot_signal
                    .peek()
                    .as_ref()
                    .is_some_and(|c| c.scryfall_data.id == new_id);
                if !holds_new {
                    stale();
                    return;
                }
                let card_name = old_card.scryfall_data.name.clone();
                let old_id = old_card.scryfall_data.id;
                let id = Opdate::Set(Some(old_id));
                let request = match slot {
                    CommandZoneSlot::Commander => {
                        HttpUpdateDeckProfile::builder().commander_id(id).build()
                    }
                    CommandZoneSlot::Partner => HttpUpdateDeckProfile::builder()
                        .partner_commander_id(id)
                        .build(),
                    CommandZoneSlot::Background => {
                        HttpUpdateDeckProfile::builder().background_id(id).build()
                    }
                    CommandZoneSlot::SignatureSpell => HttpUpdateDeckProfile::builder()
                        .signature_spell_id(id)
                        .build(),
                };

                // Optimistic like the deck-card printing arm: restore the
                // pinned slot (the featured strip updates in place).
                slot_signal.set(Some(old_card));
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
                toast.info(format!("{card_name} printing restored"), short);

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            usage_buffer.peek().report_error(
                                screen::DECK_CARD_VIEW,
                                component::NONE,
                                "undo",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                            return;
                        }
                    };
                    if let Err(e) = client()
                        .update_deck_profile(deck_id, &request, &session)
                        .await
                    {
                        usage_buffer.peek().report_error(
                            screen::DECK_CARD_VIEW,
                            component::NONE,
                            "undo",
                            &e,
                        );
                        toast.error(e.to_user_message(), ToastOptions::default());
                    }
                });
            }
        }
    };

    // Featured strip: the resolved command-zone cards role-labeled, then MVPs
    // in star order (mainboard only). Reads the live signals, so starring a
    // card updates the strip in place.
    let featured_cards: Vec<(Card, String)> = {
        let commander_role = if is_oathbreaker() {
            "Oathbreaker"
        } else {
            "Commander"
        };
        let mut mvps: Vec<DeckEntry> = deck_entries()
            .into_iter()
            .filter(|e| e.deck_card.mvp_at.is_some() && matches!(e.deck_card.board, Board::Deck))
            .collect();
        mvps.sort_by_key(|e| e.deck_card.mvp_at);
        [
            (commander_card(), commander_role),
            (partner_card(), "Partner"),
            (background_card(), "Background"),
            (signature_spell_card(), "Signature spell"),
        ]
        .into_iter()
        .filter_map(|(card, role)| card.map(|c| (c, role.to_string())))
        .chain(mvps.into_iter().map(|e| (e.card, "MVP".to_string())))
        .collect()
    };

    // Board-chip availability: an empty maybeboard/sideboard grays its chip
    // out (still un-toggleable if it was on when its last card left, so the
    // user is never stuck with a selected ghost).
    let has_maybe = deck_entries()
        .iter()
        .any(|e| matches!(e.deck_card.board, Board::Maybeboard));
    let has_side = deck_entries()
        .iter()
        .any(|e| matches!(e.deck_card.board, Board::Sideboard));

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Deck Cards", hint: deck_cards_hint_open }

                div { class: "screen-content",

                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    // Deck identity — name + format/power/tag chips in one
                    // wrapping row, echoing the zite share page (same stat-chip
                    // accents).
                    if !deck_name().is_empty() {
                        div { class: "deck-cards-header row-enter",
                            span { class: "deck-cards-header-name", "{deck_name}" }
                            if let Some(fmt) = deck_format() {
                                span { class: "stat-chip stat-chip-format", "{fmt.display_name()}" }
                            }
                            if let Some(pl) = deck_power() {
                                span { class: "stat-chip stat-chip-power", "{pl.display_name()}" }
                            }
                            for tag in deck_tags() {
                                span { key: "{tag}", class: "stat-chip stat-chip-tag", "{deck_tag_label(&tag)}" }
                            }
                            for tag in deck_other_tags() {
                                span { key: "{tag}", class: "stat-chip stat-chip-other", "{tag.display_name()}" }
                            }
                        }
                    }

                    FeaturedCards {
                        cards: featured_cards.clone(),
                        on_tap: move |card: Card| {
                            let mut preview = preview_card;
                            preview.set(Some((card.scryfall_data, 0)));
                        },
                    }

                    // Quick-add: type a name, add straight to the mainboard.
                    QuickAdd { deck_id, deck_entries }

                    // Group-by row
                    div { class: "chip-row",
                        span { class: "chip-row-label", "Group by:" }
                        for option in GroupByOption::all() {
                            Chip {
                                key: "{option}",
                                selected: group_by_option() == option,
                                onclick: move |_| {
                                    should_collapse_expanded.set(true);
                                    group_by_option.set(option);
                                },
                                "{option}"
                            }
                        }
                    }

                    // Board filter row (multi-select, at least one must be on)
                    div { class: "chip-row",
                        span { class: "chip-row-label", "Boards:" }
                        Chip {
                            selected: show_deck(),
                            onclick: move |_| {
                                let new_val = !show_deck();
                                show_deck.set(new_val);
                                // If nothing is on, snap deck back on
                                if !new_val && !show_maybe() && !show_side() {
                                    show_deck.set(true);
                                }
                                should_collapse_expanded.set(true);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "Main"
                        }
                        Chip {
                            selected: show_maybe(),
                            disabled: !has_maybe && !show_maybe(),
                            onclick: move |_| {
                                let new_val = !show_maybe();
                                show_maybe.set(new_val);
                                if !show_deck() && !new_val && !show_side() {
                                    show_deck.set(true);
                                }
                                should_collapse_expanded.set(true);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "Maybe"
                        }
                        Chip {
                            selected: show_side(),
                            disabled: !has_side && !show_side(),
                            onclick: move |_| {
                                let new_val = !show_side();
                                show_side.set(new_val);
                                if !show_deck() && !show_maybe() && !new_val {
                                    show_deck.set(true);
                                }
                                should_collapse_expanded.set(true);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "Side"
                        }
                    }

                    // Show toggles row
                    div { class: "chip-row",
                        span { class: "chip-row-label", "Show:" }
                        Chip {
                            selected: show_tokens(),
                            onclick: move |_| show_tokens.set(!show_tokens()),
                            "Tokens"
                        }
                        Chip {
                            selected: show_command_zone(),
                            onclick: move |_| show_command_zone.set(!show_command_zone()),
                            "Command zone"
                        }
                        Chip {
                            selected: show_row_art(),
                            onclick: move |_| show_row_art.set(!show_row_art()),
                            "Art"
                        }
                    }

                    if !deck_loaded() {
                        DeckCardListSkeleton {}
                    }

                    // Token list
                    if show_tokens() {
                        {
                            let sorted_tokens: Option<Vec<Card>> = tokens_resource().and_then(|r| r.ok()).map(|t| {
                                let b = filter_builder.peek();
                                Cards::from(t).sorted(b.sort().unwrap_or(CardSortKey::Name), b.ascending()).into()
                            });
                            rsx! {
                                if let Some(tokens) = sorted_tokens {
                                    if !tokens.is_empty() {
                                        div { class: "card-group row-enter",
                                            div { class: "card-group-header", "Tokens ({tokens.len()})" }
                                            for token in tokens.iter() {
                                                CardRow {
                                                    card: token.clone(),
                                                    qty: 1,
                                                    expanded_card,
                                                    preview_card,
                                                    preview_dismissing,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Maybeboard section
                    {
                        let mut mb_entries: Vec<DeckEntry> = deck_entries()
                            .into_iter()
                            .filter(|e| e.deck_card.board.is_maybeboard())
                            .collect();
                        {
                            let b = filter_builder.peek();
                            // Honor the active card filter here too, matching the
                            // mainboard pipeline (Effect 2) — not just sort.
                            if !b.is_empty()
                                && let Ok(criteria) = b.build_criteria()
                            {
                                mb_entries.retain(|e| criteria.matches(&e.card));
                            }
                            sort_deck_entries(&mut mb_entries, b.sort().unwrap_or(CardSortKey::Name), b.ascending());
                        }
                        let mb_count = mb_entries.len();
                        rsx! {
                            if show_maybe() && !mb_entries.is_empty() {
                                div { class: "card-group row-enter",
                                    div { class: "card-group-header", "Maybeboard ({mb_count})" }
                                    for entry in mb_entries.iter() {
                                        {
                                            let card_id = entry.card.scryfall_data.id;
                                            let is_basic_land = entry.card.scryfall_data.is_basic_land();
                                            let qty = *entry.deck_card.quantity;
                                            rsx! {
                                                CardRow {
                                                    card: entry.card.clone(),
                                                    qty,
                                                    expanded_card,
                                                    preview_card,
                                                    preview_dismissing,
                                                    on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land, true),
                                                    on_move_to: move |target: Board| move_to_board(card_id, target, true),
                                                    current_board: Some(Board::Maybeboard),
                                                    on_printing: move |card: Card| {
                                                        command_zone_slot.set(None);
                                                        printing_sheet_card.set(Some(card));
                                                        printing_sheet_open.set(true);
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Sideboard section
                    {
                        let mut sb_entries: Vec<DeckEntry> = deck_entries()
                            .into_iter()
                            .filter(|e| e.deck_card.board.is_sideboard())
                            .collect();
                        {
                            let b = filter_builder.peek();
                            // Honor the active card filter here too, matching the
                            // mainboard pipeline (Effect 2) — not just sort.
                            if !b.is_empty()
                                && let Ok(criteria) = b.build_criteria()
                            {
                                sb_entries.retain(|e| criteria.matches(&e.card));
                            }
                            sort_deck_entries(&mut sb_entries, b.sort().unwrap_or(CardSortKey::Name), b.ascending());
                        }
                        let sb_count = sb_entries.len();
                        rsx! {
                            if show_side() && !sb_entries.is_empty() {
                                div { class: "card-group row-enter",
                                    div { class: "card-group-header", "Sideboard ({sb_count})" }
                                    for entry in sb_entries.iter() {
                                        {
                                            let card_id = entry.card.scryfall_data.id;
                                            let is_basic_land = entry.card.scryfall_data.is_basic_land();
                                            let qty = *entry.deck_card.quantity;
                                            rsx! {
                                                CardRow {
                                                    card: entry.card.clone(),
                                                    qty,
                                                    expanded_card,
                                                    preview_card,
                                                    preview_dismissing,
                                                    on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land, true),
                                                    on_move_to: move |target: Board| move_to_board(card_id, target, true),
                                                    current_board: Some(Board::Sideboard),
                                                    on_printing: move |card: Card| {
                                                        command_zone_slot.set(None);
                                                        printing_sheet_card.set(Some(card));
                                                        printing_sheet_open.set(true);
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if show_command_zone() {
                        // Pinned commander/oathbreaker group (includes partner if present)
                        if displayed_commander().is_some() || displayed_partner().is_some() {
                            div { class: "card-group row-enter",
                                div { class: "card-group-header",
                                    if is_oathbreaker() {
                                        "Oathbreaker"
                                    } else if displayed_partner().is_some() {
                                        "Commanders"
                                    } else {
                                        "Commander"
                                    }
                                }
                                if let Some(cmd) = displayed_commander() {
                                    CardRow {
                                        card: cmd,
                                        qty: 1,
                                        expanded_card,
                                        preview_card,
                                        preview_dismissing,
                                        on_printing: move |card: Card| {
                                            command_zone_slot.set(Some(CommandZoneSlot::Commander));
                                            printing_sheet_card.set(Some(card));
                                            printing_sheet_open.set(true);
                                        },
                                    }
                                }
                                if let Some(partner) = displayed_partner() {
                                    CardRow {
                                        card: partner,
                                        qty: 1,
                                        expanded_card,
                                        preview_card,
                                        preview_dismissing,
                                        on_printing: move |card: Card| {
                                            command_zone_slot.set(Some(CommandZoneSlot::Partner));
                                            printing_sheet_card.set(Some(card));
                                            printing_sheet_open.set(true);
                                        },
                                    }
                                }
                            }
                        }

                        // Pinned background group
                        if let Some(bg) = displayed_background() {
                            div { class: "card-group row-enter",
                                div { class: "card-group-header", "Background" }
                                CardRow {
                                    card: bg,
                                    qty: 1,
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_printing: move |card: Card| {
                                        command_zone_slot.set(Some(CommandZoneSlot::Background));
                                        printing_sheet_card.set(Some(card));
                                        printing_sheet_open.set(true);
                                    },
                                }
                            }
                        }

                        // Pinned signature spell (Oathbreaker only)
                        if let Some(spell) = displayed_signature_spell() {
                            div { class: "card-group row-enter",
                                div { class: "card-group-header", "Signature spell" }
                                CardRow {
                                    card: spell,
                                    qty: 1,
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_printing: move |card: Card| {
                                        command_zone_slot.set(Some(CommandZoneSlot::SignatureSpell));
                                        printing_sheet_card.set(Some(card));
                                        printing_sheet_open.set(true);
                                    },
                                }
                            }
                        }
                    }

                    // Card groups (active deck cards only)
                    if show_deck() { for group in displayed_groups() {
                        {
                            let qty_count: i32 = group.cards.iter()
                                .map(|c| row_meta.read().get(&c.scryfall_data.id).map_or(1, |m| m.qty))
                                .sum();
                            rsx! {
                        div { class: "card-group row-enter",
                            div { class: "card-group-header",
                                "{group.label} ({qty_count})"
                            }
                            for card in group.cards.iter() {
                                {
                                    let card_id = card.scryfall_data.id;
                                    let is_basic_land = card.scryfall_data.is_basic_land();
                                    let meta = row_meta.read().get(&card_id).copied();
                                    let qty = meta.map_or(1, |m| m.qty);
                                    let card_board = meta.map(|m| m.board);
                                    let is_mvp = meta.is_some_and(|m| m.mvp);
                                    rsx! {
                                        CardRow {
                                            card: card.clone(),
                                            qty,
                                            expanded_card,
                                            preview_card,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land, true),
                                            on_move_to: move |target: Board| move_to_board(card_id, target, true),
                                            current_board: card_board,
                                            on_printing: move |card: Card| {
                                                command_zone_slot.set(None);
                                                printing_sheet_card.set(Some(card));
                                                printing_sheet_open.set(true);
                                            },
                                            mvp: Some(is_mvp),
                                            on_toggle_mvp: move |_| toggle_mvp(card_id, is_mvp),
                                        }
                                    }
                                }
                            }
                        }
                            }
                        }
                    }
                    }

                    // Lands section — pinned at the bottom of the mainboard, out of
                    // the group-by pipeline so it reads the same in every grouping
                    // mode. Hide lands via the card filter's Land type exclusion.
                    if show_deck() && !displayed_lands().is_empty() {
                        {
                            let lands = displayed_lands();
                            let qty_count: i32 = lands.iter()
                                .map(|c| row_meta.read().get(&c.scryfall_data.id).map_or(1, |m| m.qty))
                                .sum();
                            rsx! {
                                div { class: "card-group row-enter",
                                    div { class: "card-group-header", "Lands ({qty_count})" }
                                    for card in lands.iter() {
                                        {
                                            let card_id = card.scryfall_data.id;
                                            let is_basic_land = card.scryfall_data.is_basic_land();
                                            let meta = row_meta.read().get(&card_id).copied();
                                            let qty = meta.map_or(1, |m| m.qty);
                                            let card_board = meta.map(|m| m.board);
                                            let is_mvp = meta.is_some_and(|m| m.mvp);
                                            rsx! {
                                                CardRow {
                                                    card: card.clone(),
                                                    qty,
                                                    expanded_card,
                                                    preview_card,
                                                    preview_dismissing,
                                                    on_qty_change: move |delta: i32| change_quantity(card_id, delta, is_basic_land, true),
                                                    on_move_to: move |target: Board| move_to_board(card_id, target, true),
                                                    current_board: card_board,
                                                    on_printing: move |card: Card| {
                                                        command_zone_slot.set(None);
                                                        printing_sheet_card.set(Some(card));
                                                        printing_sheet_open.set(true);
                                                    },
                                                    mvp: Some(is_mvp),
                                                    on_toggle_mvp: move |_| toggle_mvp(card_id, is_mvp),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if show_deck() && displayed_groups().is_empty() && displayed_lands().is_empty() && *deck_loaded.peek() {
                        p { class: "text-muted", "No cards" }
                    }

                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(crate::inbound::router::Router::AddDeckCard { deck_id });
                    },
                    "Add"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(crate::inbound::router::Router::RemoveDeckCard { deck_id });
                    },
                    "Remove"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| filters_overlay_open.set(true),
                    "Filter"
                    if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                        span { class: "filter-dot" }
                    }
                }
                // Walks the undo log newest-first; hidden while there is
                // nothing to undo (same pattern as Reset filter below).
                if !undo_log.0.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| apply_undo(),
                        "Undo"
                    }
                }
                if !filter_builder.read().is_empty() {
                    Button {
                        variant: ButtonVariant::Util,
                        class: "util-btn-clear",
                        onclick: move |_| {
                            filter_builder.write().clear();
                            should_collapse_expanded.set(true);
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                            toast.info(
                                "Filter reset".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        },
                        "Reset filter"
                    }
                }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }

            HintDialog {
                open: deck_cards_hint_open,
                title: "Browsing your deck",
                HintLine {
                    "Tap a card's row to expand it for its details, quantity, printings, and more."
                }
                HintBullets {
                    HintBullet {
                        HintColored { color: "--accent-primary", "Keywords" }
                        " are chips. Tap one for a reminder of what it does."
                    }
                    HintBullet {
                        HintColored { color: "--accent-secondary", "Card roles" }
                        " show what a card does; tap one for its oracle tags."
                    }
                    HintBullet {
                        HintKey { color: "--color-success", "Group by" }
                        " arranges your cards by type, mana value, color, or card role."
                    }
                    HintBullet {
                        HintKey { color: "--accent-primary", "Boards" }
                        " chooses which boards are listed: main, maybe, or side."
                    }
                    HintBullet {
                        HintKey { color: "--accent-tertiary", "Show" }
                        " reveals tokens and the command zone."
                    }
                    HintBullet {
                        HintKey { color: "--color-warning", "Star" }
                        " marks a deck MVP: up to three cards that define this deck. Zwipe leans your suggestions toward them."
                    }
                }
            }

            if let Some(card) = printing_sheet_card() {
                PrintingSheet {
                    host_screen: screen::DECK_CARD_VIEW,
                    card: card.clone(),
                    open: printing_sheet_open,
                    on_save: move |new_card: Card| {
                        let old_id = card.scryfall_data.id;
                        let new_id = new_card.scryfall_data.id;

                        match command_zone_slot() {
                            Some(slot) => {
                                // Command zone card — update deck profile
                                let id = Opdate::Set(Some(new_id));
                                let request = match slot {
                                    CommandZoneSlot::Commander => HttpUpdateDeckProfile::builder().commander_id(id).build(),
                                    CommandZoneSlot::Partner => HttpUpdateDeckProfile::builder().partner_commander_id(id).build(),
                                    CommandZoneSlot::Background => HttpUpdateDeckProfile::builder().background_id(id).build(),
                                    CommandZoneSlot::SignatureSpell => HttpUpdateDeckProfile::builder().signature_spell_id(id).build(),
                                };
                                let old_card = card.clone();
                                spawn(async move {
                                    let session_val = match session.ensure_fresh(client).await {
                                        Ok(session_val) => session_val,
                                        Err(_) => return,
                                    };

                                    if client().update_deck_profile(deck_id, &request, &session_val).await.is_ok() {
                                        match slot {
                                            CommandZoneSlot::Commander => commander_card.set(Some(new_card.clone())),
                                            CommandZoneSlot::Partner => partner_card.set(Some(new_card.clone())),
                                            CommandZoneSlot::Background => background_card.set(Some(new_card.clone())),
                                            CommandZoneSlot::SignatureSpell => signature_spell_card.set(Some(new_card.clone())),
                                        }
                                        undo_log.push(UndoAction::CommandZonePrintingChanged { slot, old_card, new_id });
                                        printing_sheet_card.set(Some(new_card));
                                        let next = filter_reset_counter() + 1;
                                        filter_reset_counter.set(next);
                                    }
                                });
                            }
                            None => {
                                // Regular deck card — update deck card
                                let request = HttpPatchDeckCard::with_printing(&new_id.to_string());
                                let old_card = card.clone();
                                spawn(async move {
                                    let session_val = match session.ensure_fresh(client).await {
                                        Ok(session_val) => session_val,
                                        Err(_) => return,
                                    };

                                    if client().update_deck_card(deck_id, old_id, &request, &session_val).await.is_ok() {
                                        if let Some(entry) = deck_entries
                                            .write()
                                            .iter_mut()
                                            .find(|e| e.card.scryfall_data.id == old_id)
                                        {
                                            entry.card = new_card.clone();
                                            entry.deck_card.scryfall_data_id = new_id;
                                        }
                                        undo_log.push(UndoAction::PrintingChanged { old_card, new_id });
                                        printing_sheet_card.set(Some(new_card));
                                        let next = filter_reset_counter() + 1;
                                        filter_reset_counter.set(next);
                                    }
                                });
                            }
                        }
                    },
                }
            }

            if otag_examples_open() {
                OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
            }
            }
    }
}
