use super::components::card_info::{CardInfoDisplay, CardRulesDialog, CardSkeleton, RulesButton};
use crate::inbound::components::chip::Chip;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
            hint_dialog::{
                HintBullet, HintBullets, HintColored, HintDialog, HintLine, use_one_time_hint,
            },
            interactions::swipe::{SwipeStack, config::SwipeConfig, direction::Direction},
            telemetry::{flush_loop::flush_once, usage_buffer::UsageBuffer},
        },
        screens::deck::card::{
            components::{
                action_history::{
                    AddAction, CARDS_WARNING_THRESHOLD, MAX_CARDS_IN_STACK, MaybeboardAction,
                    StackAction,
                },
                add_stack_cache::{AddStackCache, ParkedStack},
                card_stack::{CardStack, use_card_stack},
                flippable_card_image::reset_image_ease,
            },
            filter::card_filter_sheet::CardFilterSheet,
        },
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{
            get_deck::ClientGetDeck, search_deck_cards::ClientSearchDeckCards,
            skip_deck_card::ClientSkipDeckCard,
        },
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
            update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::{budget_tier, card_price, mainboard_total_price};
use zwipe_core::domain::deck::{Board, DeckEntry, format::Format};
use zwipe_core::domain::user::models::hints::HINT_ADD_DECK_CARDS;
use zwipe_core::domain::{
    card::{
        Card,
        scryfall_data::{
            ImageSize,
            colors::{Color, Colors},
        },
        search_card::{
            card_filter::builder::CardQueryBuilder,
            card_type::CardType,
            cards::Cards,
        },
    },
    deck::Quantity,
};
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpUpdateDeckCard};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum AddSource {
    #[default]
    Search,
    Maybeboard,
}

/// Applies the "lands already at target" state to the search filter: adds
/// `CardType::Land` to the excludes and drops it from either include list, so we
/// never end up with an include+exclude clash on Land (which serves zero cards).
/// Merges with the user's existing card-type filters. Applied once on entry to
/// the add screen when the deck already meets its land target.
fn ensure_lands_excluded(mut filter_builder: Signal<CardQueryBuilder>) {
    let mut fb = filter_builder.write();

    // Read current state into owned values so the immutable borrows drop before
    // we mutate below.
    let contains_any = fb.card_type_contains_any().map(<[CardType]>::to_vec);
    let contains_all = fb.card_type_contains_all().map(<[CardType]>::to_vec);
    let mut excluded = fb
        .card_type_excludes_any()
        .map(<[CardType]>::to_vec)
        .unwrap_or_default();

    // Drop Land from any include list — including and excluding Land at once
    // matches nothing. Setting an empty vec clears the filter.
    if let Some(includes) = contains_any {
        let kept: Vec<CardType> = includes.into_iter().filter(|t| *t != CardType::Land).collect();
        fb.set_card_type_contains_any(kept);
    }
    if let Some(includes) = contains_all {
        let kept: Vec<CardType> = includes.into_iter().filter(|t| *t != CardType::Land).collect();
        fb.set_card_type_contains_all(kept);
    }

    // Add Land to the excludes if not already present.
    if !excluded.contains(&CardType::Land) {
        excluded.push(CardType::Land);
        fb.set_card_type_excludes_any(excluded);
    }
}

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardQueryBuilder> = use_context();
    // App-scoped search stack (cards, cursor, undo history, animation) —
    // survives navigation so re-entry resumes mid-stack.
    let mut stack: CardStack<AddAction> = use_context();
    // Parked stacks for other decks; this deck's parked entry (if any) is
    // restored by the mount effect, and `use_drop` below parks on leave.
    let mut stack_cache: AddStackCache = use_context();
    let mut last_search_filter: Signal<Option<CardQueryBuilder>> = use_context();
    let is_first_run = use_hook(|| std::cell::Cell::new(true));

    // Swipe vocabulary hint: auto-opens on this user's first visit, the
    // grayed "?" in the util bar reopens it on demand.
    let swipe_hint_open = use_one_time_hint(HINT_ADD_DECK_CARDS);
    let show_rules = use_signal(|| false);

    let mut deck_cards_ids = use_signal(HashSet::<Uuid>::new);
    let mut deck_format: Signal<Option<Format>> = use_signal(|| None);
    let mut deck_color_identity: Signal<Option<Colors>> = use_signal(|| None);
    let mut deck_has_commander = use_signal(|| false);
    // Primary commander's oracle id, for keying the first-party suggestion
    // signal `(commander, card)` recorded on each add-stack swipe.
    let mut commander_oracle_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut deck_loaded = use_signal(|| false);

    // Land-count signal: the current mainboard land count and the effective
    // target (the deck's user-set override, else the format heuristic). When an
    // add pushes the count up across the target we toast once.
    let mut mainboard_land_count: Signal<i32> = use_signal(|| 0);
    let mut land_target: Signal<Option<i32>> = use_signal(|| None);

    // Budget signal: running mainboard total in the budget currency, plus the
    // budget + currency. Each add raises the running total; crossing 50/75/100%
    // toasts once.
    let mut deck_total_price: Signal<f64> = use_signal(|| 0.0);
    let mut price_budget: Signal<Option<f64>> = use_signal(|| None);
    let mut price_budget_currency: Signal<PriceCurrency> = use_signal(|| PriceCurrency::Usd);

    // Source selector: search (API) vs maybeboard (local)
    let mut add_source: Signal<AddSource> = use_signal(AddSource::default);
    // True when synergy was requested but the commander's cache is still warming
    // (server fell back to the full pool); drives the inline "warming up" note.
    let mut synergy_warming = use_signal(|| false);
    let mut mb_entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);
    // Screen-local maybeboard stack — a cycling view over `mb_entries`.
    let mut mb_stack = use_card_stack::<MaybeboardAction>();

    // Filters overlay at bottom of screen state
    let mut filters_overlay_open = use_signal(|| false);

    // Gesture start point for the end-of-stack skeleton, which accepts a
    // down-swipe to undo back into the pile (no card, so no SwipeStack).
    let mut end_swipe_start: Signal<Option<(f64, f64)>> = use_signal(|| None);

    // Reset counter for collapsing accordions and clearing search queries
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();

    // Toast once when synergy goes cold (the search fell back to the full pool).
    // Deduped via a prev-value cell so re-searches / load-more while still warming
    // don't re-fire it; only a fresh false->true transition toasts.
    let synergy_was_warming = use_hook(|| std::cell::Cell::new(false));
    use_effect(move || {
        let warming = synergy_warming();
        let was = synergy_was_warming.replace(warming);
        if warming && !was {
            toast.info(
                "Synergy warming up; showing all cards for now".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3500)),
            );
        }
    });

    // Pagination state
    let mut current_offset = use_signal(|| 0_u32);
    let mut is_loading_more = use_signal(|| false);
    let pagination_limit = 25_u32; // Matches backend default
    // Keep the buffer comfortably ahead of the STACK_DEPTH window so the
    // stack never visibly shrinks while a batch is in flight.
    let load_more_threshold = 5_usize;
    let mut pagination_exhausted = use_signal(|| false);

    let mut is_loading_cards = use_signal(|| false);

    // Park this deck's stack on leave so returning resumes mid-stack. Only a
    // served stack parks — an empty one, or one with no recorded filter,
    // refetches next time anyway.
    use_drop(move || {
        let Some(filter) = last_search_filter.peek().clone() else {
            return;
        };
        if stack.peek_is_empty() {
            return;
        }
        let (cards, index, history) = stack.park_state();
        stack_cache.park(
            deck_id,
            ParkedStack {
                cards,
                index,
                history,
                filter,
                offset: *current_offset.peek(),
                exhausted: *pagination_exhausted.peek(),
            },
        );
    });

    let current_card = move || stack.current();

    // Lands drop out of the pool once the deck's land target is met. That auto
    // exclusion is a default, not a user filter, so the dot ignores it (see the
    // filter-dot condition below).
    let lands_at_target = move || {
        land_target()
            .or_else(|| deck_format().and_then(|f| f.default_land_target()))
            .is_some_and(|t| mainboard_land_count() >= t)
    };

    // Load more cards with pagination and de-duplication
    let mut load_more_cards = move || {
        // Check if we've hit the card limit
        let current_card_count = stack.len();
        if current_card_count >= MAX_CARDS_IN_STACK {
            toast.warning(
                "Card limit reached, please refresh to continue".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
            return;
        }

        if is_loading_more() {
            return;
        }

        is_loading_more.set(true);

        let mut builder = filter_builder.read().clone();
        builder.set_is_token(false);
        builder.set_limit(pagination_limit);
        builder.set_offset(current_offset());

        let Ok(filter) = builder.build() else {
            is_loading_more.set(false);
            return;
        };

        usage_buffer().record_search();
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(_) => {
                    is_loading_more.set(false);
                    return;
                }
            };

            match client().search_deck_cards(deck_id, &filter, &session).await {
                Ok((new_cards, warming)) => {
                    synergy_warming.set(warming);
                    let existing_cards = stack.cards();
                    let deck_ids = deck_cards_ids();

                    // Get existing card IDs for de-duplication
                    let existing_ids: HashSet<Uuid> = existing_cards
                        .iter()
                        .map(|c| c.card_profile.scryfall_data_id)
                        .collect();

                    // Filter out duplicates, deck cards, and cards without images
                    let unique_new_cards: Vec<Card> = new_cards
                        .into_iter()
                        .filter(|card| {
                            !existing_ids.contains(&card.card_profile.scryfall_data_id)
                                && !card
                                    .scryfall_data
                                    .oracle_id
                                    .is_some_and(|oid| deck_ids.contains(&oid))
                                && card
                                    .scryfall_data
                                    .primary_image_url(ImageSize::Large)
                                    .is_some()
                        })
                        .collect();

                    // Append unique cards to existing list
                    if !unique_new_cards.is_empty() {
                        stack.append(unique_new_cards);

                        // Update offset for next load
                        current_offset.set(current_offset() + pagination_limit);
                    } else {
                        pagination_exhausted.set(true);
                    }

                    is_loading_more.set(false);
                }
                Err(e) => {
                    tracing::warn!("pagination load failed: {e}");
                    is_loading_more.set(false);
                }
            }
        });
    };

    // Swipe config — the stack owns its own SwipeState internally.
    // Thresholds tuned for responsive rapid swiping: a short drag (60px) OR
    // a quick flick (1.5 px/ms over the 10px minimum) commits.
    let swipe_config = SwipeConfig::new(
        vec![
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ],
        60.0, // distance threshold in px
        1.5,  // speed threshold in px/ms
    );

    // Advance past the just-committed card. The stack fires its on_swipe_*
    // callbacks after the exit transition, so by now the card is off-screen.
    // The cursor may land one past the end (empty window) — that keeps undo
    // aligned with the swipe that just committed.
    let mut advance_after_commit = move || {
        let total = stack.len();
        if !stack.advance() {
            return;
        }
        if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total)
            && total.is_multiple_of(100)
        {
            toast.info(
                "Approaching card limit, consider refreshing".to_string(),
                ToastOptions::default().duration(Duration::from_millis(2000)),
            );
        }
        if stack.index() >= total && pagination_exhausted() {
            // Past the last card with nothing left to fetch.
            toast.warning("End of results".to_string(), ToastOptions::default());
        } else if total > 0 && stack.index() + 1 >= total.saturating_sub(load_more_threshold) {
            // Within the prefetch threshold — top the stack up.
            load_more_cards();
        }
    };

    let add_card_to_deck = move |card: Card| {
        // For now, always add quantity 1 (will add quantity picker later)
        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
        let oracle_id = card.scryfall_data.oracle_id;

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(_) => {
                    if let Some(oid) = oracle_id {
                        deck_cards_ids.write().insert(oid);
                    }
                }
                Err(e) => {
                    tracing::warn!("add card to deck failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    let add_card_to_maybeboard = move |card: Card| {
        let request =
            HttpCreateDeckCard::new(&card.scryfall_data, 1, Some("maybeboard".to_string()));
        let oracle_id = card.scryfall_data.oracle_id;

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(deck_card) => {
                    if let Some(oid) = oracle_id {
                        deck_cards_ids.write().insert(oid);
                    }
                    // Keep the maybeboard source in sync so switching to it
                    // shows this card without a refetch.
                    mb_entries.write().push(DeckEntry { card, deck_card });
                }
                Err(e) => {
                    tracing::warn!("add card to maybeboard failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    let mut undo_last_action = move || {
        // Pop last action from history
        let Some(action) = stack.pop_action() else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        // Step back one card (primes the enter animation from the direction
        // the card originally exited). Fails at the first card.
        if !stack.step_back(&action) {
            toast.warning("No previous card".to_string(), ToastOptions::default());
            stack.record(action); // Put it back
            return;
        }

        // The cursor now sits on the very card the action committed — the
        // linear stack never drops swiped cards, so the card is read back
        // from the stack rather than stored in the history.
        let Some(card) = stack.current() else {
            stack.unwind_undo(action);
            return;
        };

        match action {
            AddAction::Skip => {
                // Delete the suppression row posted at swipe time.
                if let Some(oracle_id) = card.scryfall_data.oracle_id {
                    spawn(async move {
                        let session = match session.ensure_fresh(client).await {
                            Ok(session) => session,
                            Err(e) => {
                                toast.error(e.to_user_message(), ToastOptions::default());
                                return;
                            }
                        };
                        if let Err(e) = client().unskip_deck_card(deck_id, oracle_id, &session).await {
                            tracing::warn!("undo skip (unskip) failed: {e}");
                            toast.error(format!("Failed to undo skip: {}", e), ToastOptions::default());
                        }
                    });
                }
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            AddAction::Add => {
                // Need to delete from backend (undoing the add)
                let card_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;
                let was_land = card.scryfall_data.is_land();
                let was_price = card_price(&card.scryfall_data, price_budget_currency()).unwrap_or(0.0);

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            stack.unwind_undo(action); // Restore history + cursor
                            return;
                        }
                    };

                    match client().delete_deck_card(deck_id, card_id, &session).await {
                        Ok(_) => {
                            // Remove from exclusion HashSet
                            if let Some(oid) = oracle_id {
                                deck_cards_ids.write().remove(&oid);
                            }
                            if was_land {
                                mainboard_land_count.set((mainboard_land_count() - 1).max(0));
                            }
                            deck_total_price.set((deck_total_price() - was_price).max(0.0));
                            toast.success(
                                "Undid add".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo add (delete deck card) failed: {e}");
                            toast.error(format!("Failed to undo: {}", e), ToastOptions::default());
                            // Don't restore action or index - user can try again by adding the card
                        }
                    }
                });
            }
            AddAction::Maybe => {
                let card_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            stack.unwind_undo(action);
                            return;
                        }
                    };

                    match client().delete_deck_card(deck_id, card_id, &session).await {
                        Ok(_) => {
                            if let Some(oid) = oracle_id {
                                deck_cards_ids.write().remove(&oid);
                            }
                            // Mirror add_card_to_maybeboard's sync: the card
                            // is off the maybeboard again.
                            mb_entries
                                .write()
                                .retain(|e| e.card.scryfall_data.id != card_id);
                            toast.success(
                                "Undid maybeboard".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo maybeboard (delete deck card) failed: {e}");
                            toast.error(format!("Failed to undo: {}", e), ToastOptions::default());
                        }
                    }
                });
            }
        }
    };

    let mut clear_filters = move || {
        let opts = ToastOptions::default().duration(Duration::from_millis(1500));
        // Reset returns to the screen's default view. Synergy is a separate
        // mode, not a filter, so it survives the reset (a commander deck lands
        // back in synergy order). A lone sort counts as non-default, so Reset
        // fires (and clears the sort) even when no card filter is set.
        let synergy = filter_builder.peek().synergy();
        let has_sort = filter_builder.peek().sort().is_some();
        if add_source() == AddSource::Maybeboard {
            // Maybeboard default is a blank filter.
            if filter_builder.read().is_empty() && !has_sort {
                toast.warning("Filter already at default".to_string(), opts);
            } else {
                filter_builder.write().clear();
                filter_builder.write().set_synergy(synergy);
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
                toast.info("Filter reset".to_string(), opts);
            }
        } else {
            // Search default is deck context (colors + legality) in synergy
            // order, plus the automatic land exclusion once the land target is
            // met. All of that is part of the default, so it's re-applied here
            // and doesn't count toward "already at default".
            let at_default = filter_builder
                .read()
                .is_empty_ignoring_deck_context_and_auto_lands(lands_at_target());
            if at_default && !has_sort {
                toast.warning("Filter already at default".to_string(), opts);
            } else {
                filter_builder.write().clear();
                if let Some(fmt) = deck_format() {
                    filter_builder
                        .write()
                        .set_legalities_contains_any(vec![fmt.to_legality_key().to_string()]);
                }
                if let Some(colors) = deck_color_identity() {
                    filter_builder.write().set_color_identity_within(colors);
                }
                filter_builder.write().set_synergy(synergy);
                // Keep lands out if the deck is already at its land target.
                if lands_at_target() {
                    ensure_lands_excluded(filter_builder);
                }
                stack.replace(Vec::new());
                toast.info("Filter reset".to_string(), opts);
            }
        }
    };

    // Fetch deck cards on mount for filtering
    use_effect(move || {
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(_) => {
                    return;
                }
            };

            match client().get_deck(deck_id, &session).await {
                Ok(deck) => {
                    let mut ids: HashSet<_> = deck
                        .entries
                        .iter()
                        .filter_map(|entry| entry.card.scryfall_data.oracle_id)
                        .collect();
                    // Resolve command zone cards to oracle_ids for exclusion
                    // and collect color identities from commander/partner/background
                    let mut identity_colors: Vec<Color> = Vec::new();
                    for cz_id in [
                        deck.deck_profile.commander_id,
                        deck.deck_profile.partner_commander_id,
                        deck.deck_profile.background_id,
                    ]
                    .into_iter()
                    .flatten()
                    {
                        if let Ok(card) = client().get_card(cz_id).await
                            && let Some(oid) = card.scryfall_data.oracle_id
                        {
                            // Capture the primary commander's oracle id to key the
                            // suggestion signal (partner/background contribute under it).
                            if Some(cz_id) == deck.deck_profile.commander_id {
                                commander_oracle_id.set(Some(oid));
                            }
                            ids.insert(oid);
                            identity_colors
                                .extend(card.scryfall_data.color_identity.iter().cloned());
                        }
                    }
                    // Signature spell: oracle_id exclusion only (doesn't contribute to color identity)
                    if let Some(spell_id) = deck.deck_profile.signature_spell_id
                        && let Ok(card) = client().get_card(spell_id).await
                        && let Some(oid) = card.scryfall_data.oracle_id
                    {
                        ids.insert(oid);
                    }
                    deck_cards_ids.set(ids);

                    // Collect maybeboard entries for the maybeboard source mode
                    let mb: Vec<DeckEntry> = deck
                        .entries
                        .iter()
                        .filter(|e| e.deck_card.board.is_maybeboard())
                        .cloned()
                        .collect();
                    mb_entries.set(mb);

                    deck_has_commander.set(deck.deck_profile.commander_id.is_some());
                    // Default Synergy ON when the deck has a commander (the
                    // curated pool); OFF otherwise. Set once on load — the user
                    // flips it via the toggle, and the server falls back to the
                    // full pool if the commander's synergy cache is still warming.
                    filter_builder
                        .write()
                        .set_synergy(deck.deck_profile.commander_id.is_some());
                    deck_loaded.set(true);

                    // Seed the land signal: count mainboard lands (quantity-aware,
                    // MDFC land faces included via is_land's combined type line),
                    // and resolve the target from the deck override or the format.
                    let land_count: i32 = deck
                        .entries
                        .iter()
                        .filter(|e| e.deck_card.board.is_active() && e.card.scryfall_data.is_land())
                        .map(|e| *e.deck_card.quantity)
                        .sum();
                    mainboard_land_count.set(land_count);
                    // Explicit target only — no land toasts unless the user set one.
                    land_target.set(deck.deck_profile.land_target);
                    // If the deck already meets its land target (explicit override
                    // or the format default), start with lands excluded from the
                    // swipe pool. The auto-serve reset below re-fetches, so no
                    // separate counter bump is needed here.
                    let effective_target = deck
                        .deck_profile
                        .land_target
                        .or_else(|| deck.deck_profile.format.and_then(|f| f.default_land_target()));
                    if let Some(t) = effective_target
                        && land_count >= t
                    {
                        ensure_lands_excluded(filter_builder);
                    }

                    // Seed the budget signal: running total in the budget
                    // currency, plus the budget + currency from the profile.
                    let currency = deck
                        .deck_profile
                        .price_target_currency
                        .unwrap_or(PriceCurrency::Usd);
                    deck_total_price.set(mainboard_total_price(&deck.entries, currency));
                    price_budget.set(deck.deck_profile.price_target);
                    price_budget_currency.set(currency);

                    // Pre-populate format filter from deck
                    if let Some(fmt) = deck.deck_profile.format {
                        deck_format.set(Some(fmt));
                        if filter_builder.peek().legalities_contains_any().is_none() {
                            filter_builder.write().set_legalities_contains_any(vec![
                                fmt.to_legality_key().to_string(),
                            ]);
                        }

                        // Pre-populate color identity filter from commander
                        if fmt.checks_color_identity() && deck.deck_profile.commander_id.is_some() {
                            identity_colors.sort();
                            identity_colors.dedup();
                            let colors: Colors = identity_colors.into();
                            deck_color_identity.set(Some(colors.clone()));
                            if filter_builder.peek().color_identity_within().is_none() {
                                filter_builder.write().set_color_identity_within(colors);
                            }
                        }
                    }

                    // Auto-serve: with deck context now populated, an empty
                    // stack can be filled immediately — the deck-aware search
                    // serves the default filter synergy-ordered, 25 a page.
                    // A non-empty stack is a preserved session; leave it be.
                    if stack.peek_is_empty() {
                        let current = *filter_reset_counter.peek();
                        filter_reset_counter.set(current + 1);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "deck card filter fetch failed, continuing without filtering: {e}"
                    );
                }
            }
        });
    });

    use_effect(move || {
        let _ = filter_reset_counter();

        let first = is_first_run.get();
        is_first_run.set(false);

        let mut builder = filter_builder.peek().clone();
        builder.set_is_token(false);
        builder.set_limit(pagination_limit);
        builder.set_offset(0);

        if first {
            // ── Initial mount ─────────────────────────────────────────
            // Un-park this deck's stack if its filter still matches.
            if let Some(parked) = stack_cache.take(deck_id) {
                let mut prev_b = parked.filter.clone();
                prev_b.set_is_token(false);
                prev_b.set_limit(pagination_limit);
                prev_b.set_offset(0);
                if prev_b == builder && !parked.cards.is_empty() {
                    current_offset.set(parked.offset);
                    pagination_exhausted.set(parked.exhausted);
                    last_search_filter.set(Some(parked.filter.clone()));
                    stack.restore(parked.cards, parked.index, parked.history);
                    return;
                }
                // Filter changed since parking — fall through to a fresh fetch.
            }
        }

        // ── Clear and re-fetch ────────────────────────────────────────
        // Reaches here when:
        //   - explicit user action (refresh / apply filter), OR
        //   - initial mount with a different/new filter, OR
        //   - initial mount with no existing cards
        stack.reset();
        // Fresh results ease in again, even previously seen images.
        reset_image_ease();
        last_search_filter.set(None);
        current_offset.set(0);
        pagination_exhausted.set(false);

        // Gate (Search mode, once the deck is loaded): only auto-serve a
        // meaningful query. A commander format *with* a commander pulls
        // synergy suggestions from cache; otherwise we need some intent (a
        // filter, a sort, or synergy on). Only a truly blank query on a deck
        // with no commander is left unserved, nudging the user to filter — an
        // empty screen with no prompt reads as "no cards exist".
        if *deck_loaded.peek() && matches!(*add_source.peek(), AddSource::Search) {
            let is_commander_format = deck_format.peek().is_some_and(|f| f.has_commander());
            let has_commander = *deck_has_commander.peek();
            let has_intent = filter_builder.peek().has_search_intent();
            let serve = if is_commander_format {
                has_commander || has_intent
            } else {
                has_intent
            };
            if !serve {
                toast.warning(
                    "Try a filter to start swiping".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(2500)),
                );
                return;
            }
        }

        // Auto-serve: a filter holding only deck context (format legality +
        // commander identity) is a valid search now — the deck-aware endpoint
        // serves it synergy-ordered. Only a truly empty builder (deck has no
        // format/commander, context not yet loaded) fails build() and keeps
        // today's empty state; the mount effect re-triggers once context lands.
        let Ok(filter) = builder.build() else {
            return;
        };

        // Peek session to avoid subscribing this effect to session changes.
        // The interval-based upkeep in Bouncer handles session refresh.
        let session_signal = session;
        let Some(session) = session.peek().clone() else {
            toast.error("Session expired".to_string(), ToastOptions::default());
            return;
        };

        is_loading_cards.set(true);

        // Snapshot the filter builder state before the async block owns context.
        let filter_snapshot = filter_builder.peek().clone();

        usage_buffer().record_search();
        spawn(async move {
            // Flush buffered usage (signals feed synergy ordering) before the
            // refetch. Skips post directly at swipe time.
            flush_once(&usage_buffer(), &client, &session_signal).await;
            match client().search_deck_cards(deck_id, &filter, &session).await {
                Ok((cards_from_search, warming)) => {
                    synergy_warming.set(warming);
                    let deck_ids = deck_cards_ids();
                    stack.replace(
                        cards_from_search
                            .into_iter()
                            .filter(|card| {
                                card.scryfall_data
                                    .primary_image_url(ImageSize::Large)
                                    .is_some()
                                    && !card
                                        .scryfall_data
                                        .oracle_id
                                        .is_some_and(|oid| deck_ids.contains(&oid))
                            })
                            .collect(),
                    );
                    // Record the filter that produced these results.
                    last_search_filter.set(Some(filter_snapshot));
                    // Set offset for next page
                    current_offset.set(pagination_limit);
                    is_loading_cards.set(false);
                }
                Err(e) => {
                    tracing::warn!("card search failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                    is_loading_cards.set(false);
                }
            }
        });
    });

    // Maybeboard filter effect — applies client-side filtering + sorting
    use_effect(move || {
        let _ = add_source();
        let _ = filter_reset_counter();

        if add_source() != AddSource::Maybeboard {
            return;
        }

        let entries = mb_entries.peek().clone();
        let builder = filter_builder.peek().clone();

        let cards: Vec<Card> = entries.iter().map(|e| e.card.clone()).collect();
        // In-memory path: bare criteria (no pagination), then the builder's sort.
        let filtered = if builder.is_empty() {
            cards
        } else {
            match builder.build_criteria() {
                Ok(criteria) => Cards::from(cards).matching(&criteria).into(),
                Err(_) => cards,
            }
        };
        let filtered: Vec<Card> = Cards::from(filtered)
            .sorted(builder.sort(), builder.ascending())
            .into();
        mb_stack.replace(filtered);
    });

    let mb_current_card = move || mb_stack.current_wrapping();

    let mut mb_promote_to_deck = move |card: Card| {
        let scryfall_data_id = card.scryfall_data.id;
        let request = HttpUpdateDeckCard::new(None, Some("deck".to_string()));

        // Optimistic: remove from maybeboard lists
        mb_entries
            .write()
            .retain(|e| e.card.scryfall_data.id != scryfall_data_id);
        mb_stack.remove_current();
        // Also add to deck exclusion set so search mode won't show it
        if let Some(oid) = card.scryfall_data.oracle_id {
            deck_cards_ids.write().insert(oid);
        }

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            if let Err(e) = client()
                .update_deck_card(deck_id, scryfall_data_id, &request, &session)
                .await
            {
                tracing::warn!("promote to deck failed: {e}");
                toast.error(e.to_user_message(), ToastOptions::default());
            }
        });
    };

    let mut mb_undo_last_action = move || {
        let Some(action) = mb_stack.pop_action() else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        mb_stack.prime_entering(action.exited());

        match action {
            MaybeboardAction::Skip => {
                if !mb_stack.retreat_wrapping() {
                    mb_stack.cancel_entering();
                    return;
                }
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            MaybeboardAction::Promote { card } => {
                let card = *card;
                mb_stack.insert_current(card.clone());

                let scryfall_data_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;
                let request = HttpUpdateDeckCard::new(None, Some("maybeboard".to_string()));

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            mb_stack.cancel_entering();
                            return;
                        }
                    };

                    match client()
                        .update_deck_card(deck_id, scryfall_data_id, &request, &session)
                        .await
                    {
                        Ok(_) => {
                            if let Some(oid) = oracle_id {
                                deck_cards_ids.write().remove(&oid);
                            }
                            // Re-add to mb_entries source of truth
                            mb_entries.write().push(DeckEntry {
                                card,
                                deck_card: zwipe_core::domain::deck::DeckCard {
                                    deck_id,
                                    scryfall_data_id,
                                    oracle_id: oracle_id.unwrap_or_default(),
                                    quantity: Quantity::one(),
                                    board: Board::Maybeboard,
                                },
                            });
                            toast.success(
                                "Undid move to deck".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo promote failed: {e}");
                            toast.error(format!("Failed to undo: {}", e), ToastOptions::default());
                        }
                    }
                });
            }
        }
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                ScreenHeader { title: "Add Deck Cards", hint: swipe_hint_open }

                div { class: "screen-content card-swipe content-enter",

                // Source selector chips
                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    div { class: "chip-row",
                        span { class: "chip-row-label", "From:" }
                        for (label, variant) in [("Search", AddSource::Search), ("Maybeboard", AddSource::Maybeboard)] {
                            Chip {
                                selected: add_source() == variant,
                                onclick: move |_| {
                                    if add_source() == variant { return; }

                                    match variant {
                                        AddSource::Maybeboard => {
                                            // If filter is just deck-context defaults, blank it
                                            if filter_builder.read().is_empty_ignoring_deck_context() {
                                                filter_builder.write().clear();
                                            }
                                        }
                                        AddSource::Search => {
                                            // If filter is truly blank, re-apply deck-context defaults
                                            if filter_builder.read().is_empty() {
                                                if let Some(fmt) = deck_format() {
                                                    filter_builder.write().set_legalities_contains_any(
                                                        vec![fmt.to_legality_key().to_string()]
                                                    );
                                                }
                                                if let Some(colors) = deck_color_identity() {
                                                    filter_builder.write().set_color_identity_within(colors);
                                                }
                                            }
                                        }
                                    }

                                    add_source.set(variant);
                                    mb_stack.rewind();
                                    let current = *filter_reset_counter.peek();
                                    filter_reset_counter.set(current + 1);
                                },
                                "{label}"
                            }
                        }
                        // Synergy ON/OFF — constrains the stack to the commander's
                        // synergy pool, then sorts within it. Search source +
                        // commander only (no commander = nothing to constrain to).
                        // Pinned right (margin-left:auto) so it reads as its own
                        // control, not part of the "From:" source group.
                        if add_source() == AddSource::Search && deck_has_commander() {
                            div { style: "margin-left: auto;",
                                Chip {
                                    selected: filter_builder.read().synergy(),
                                    onclick: move |_| {
                                        let on = !filter_builder.peek().synergy();
                                        filter_builder.write().set_synergy(on);
                                        let current = *filter_reset_counter.peek();
                                        filter_reset_counter.set(current + 1);
                                    },
                                    "Synergy"
                                }
                            }
                        }
                    }
                }

                div { class : "form-container",
                    if add_source() == AddSource::Search {
                        // Search mode (existing behavior)
                        if !stack.is_empty() && stack.index() < stack.len() {
                            SwipeStack {
                                cards: stack.window(),
                                config: swipe_config,
                                entering: stack.entering(),
                                on_swipe_left: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Left);
                                    usage_buffer().record_signal(commander_oracle_id(), card.scryfall_data.oracle_id, Direction::Left);
                                    // Post the durable skip immediately — a buffered
                                    // skip is lost to a quick app kill.
                                    if let Some(oracle_id) = card.scryfall_data.oracle_id {
                                        spawn(async move {
                                            let Ok(session) = session.ensure_fresh(client).await else {
                                                return;
                                            };
                                            if let Err(e) = client().skip_deck_card(deck_id, oracle_id, &session).await {
                                                tracing::warn!("skip post failed: {e}");
                                            }
                                        });
                                    }
                                    stack.record(AddAction::Skip);
                                    toast.info("Skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    advance_after_commit();
                                },
                                on_swipe_right: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Right);
                                    usage_buffer().record_signal(commander_oracle_id(), card.scryfall_data.oracle_id, Direction::Right);
                                    stack.record(AddAction::Add);
                                    let added_land = card.scryfall_data.is_land();
                                    let added_price = card_price(&card.scryfall_data, price_budget_currency()).unwrap_or(0.0);
                                    add_card_to_deck(card);
                                    toast.success("Added to deck".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    if added_land {
                                        let prev = mainboard_land_count();
                                        let now = prev + 1;
                                        mainboard_land_count.set(now);
                                        // Toast only on the upward crossing — debounced so
                                        // every further land doesn't re-fire.
                                        if let Some(target) = land_target()
                                            && prev < target && now >= target
                                        {
                                            toast.info(
                                                format!("Land target reached ({target})"),
                                                ToastOptions::default().duration(Duration::from_millis(2500)),
                                            );
                                            // We don't touch the filter mid-session — a
                                            // refetch here would reset the swipe stack and
                                            // lose the user's spot. Lands are excluded only
                                            // on the next entry to the add screen (see the
                                            // deck-load block).
                                        }
                                    }
                                    // Budget band: raise the running total, toast when it
                                    // crosses into a higher 50/75/100% band (exact percentage).
                                    if let Some(budget) = price_budget().filter(|b| *b > 0.0) {
                                        let before = deck_total_price();
                                        let after = before + added_price;
                                        deck_total_price.set(after);
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
                                    }
                                    advance_after_commit();
                                },
                                on_swipe_up: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Up);
                                    usage_buffer().record_signal(commander_oracle_id(), card.scryfall_data.oracle_id, Direction::Up);
                                    stack.record(AddAction::Maybe);
                                    add_card_to_maybeboard(card);
                                    toast.info("Added to maybeboard".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    advance_after_commit();
                                },
                                on_swipe_down: move |_card: Card| {
                                    usage_buffer().record_swipe(Direction::Down);
                                    undo_last_action();
                                },
                            }

                            if let Some(card) = current_card() {
                                CardInfoDisplay { card: card.clone() }
                                CardRulesDialog { open: show_rules, card }
                            }
                        } else if !stack.is_empty() {
                            // Cursor sits one past the last card. A prefetch
                            // may still be topping the stack up; a dry end
                            // keeps the skeleton down-swipeable so undo leads
                            // back into the pile.
                            if is_loading_more() {
                                CardSkeleton { is_loading: true }
                            } else {
                                div {
                                    style: "display: flex; flex-direction: column; flex: 1; min-height: 0; width: 100%;",
                                    ontouchstart: move |e: Event<TouchData>| {
                                        if let Some(t) = e.touches().into_iter().next() {
                                            let p = t.client_coordinates();
                                            end_swipe_start.set(Some((p.x, p.y)));
                                        }
                                    },
                                    ontouchend: move |e: Event<TouchData>| {
                                        let Some((sx, sy)) = end_swipe_start() else {
                                            return;
                                        };
                                        end_swipe_start.set(None);
                                        if let Some(t) = e.touches_changed().into_iter().next() {
                                            let p = t.client_coordinates();
                                            if p.y - sy >= 60.0 && (p.y - sy).abs() > (p.x - sx).abs() {
                                                undo_last_action();
                                            }
                                        }
                                    },
                                    onmousedown: move |e: Event<MouseData>| {
                                        let p = e.client_coordinates();
                                        end_swipe_start.set(Some((p.x, p.y)));
                                    },
                                    onmouseup: move |e: Event<MouseData>| {
                                        let Some((sx, sy)) = end_swipe_start() else {
                                            return;
                                        };
                                        end_swipe_start.set(None);
                                        let p = e.client_coordinates();
                                        if p.y - sy >= 60.0 && (p.y - sy).abs() > (p.x - sx).abs() {
                                            undo_last_action();
                                        }
                                    },
                                    CardSkeleton {}
                                }
                            }
                        } else if is_loading_cards() {
                            CardSkeleton { is_loading: true }
                        } else {
                            CardSkeleton {}
                        }
                    } else {
                        // Maybeboard mode
                        if !mb_stack.is_empty() {
                            SwipeStack {
                                cards: mb_stack.window(),
                                config: swipe_config,
                                entering: mb_stack.entering(),
                                on_swipe_left: move |_card: Card| {
                                    usage_buffer().record_swipe(Direction::Left);
                                    mb_stack.record(MaybeboardAction::Skip);
                                    toast.info("Skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    mb_stack.advance_wrapping();
                                },
                                on_swipe_right: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Right);
                                    mb_stack.record(MaybeboardAction::Promote { card: Box::new(card.clone()) });
                                    mb_promote_to_deck(card);
                                    toast.success("Moved to deck".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                },
                                on_swipe_up: move |_card: Card| {
                                    usage_buffer().record_swipe(Direction::Up);
                                    toast.info("Already on maybeboard".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                },
                                on_swipe_down: move |_card: Card| {
                                    usage_buffer().record_swipe(Direction::Down);
                                    mb_undo_last_action();
                                },
                            }

                            if let Some(card) = mb_current_card() {
                                CardInfoDisplay { card: card.clone() }
                                CardRulesDialog { open: show_rules, card }
                            }
                        } else {
                            CardSkeleton {}
                        }
                    }

                }
            }

            div {
                class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        // Clear auto-populated defaults so view/remove screens start fresh
                        if filter_builder.read().is_empty_ignoring_deck_context() {
                            filter_builder.write().clear();
                        }
                        navigator.go_back();
                    },
                    "Back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "Filter"
                    if !filter_builder.read().is_empty_ignoring_deck_context_and_auto_lands(lands_at_target()) || filter_builder.read().sort().is_some() {
                        span { class: "filter-dot" }
                    }
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        if add_source() == AddSource::Maybeboard {
                            // Maybeboard mode: re-fetch deck to reload maybeboard entries
                            spawn(async move {
                                let session = match session.ensure_fresh(client).await {
                                    Ok(session) => session,
                                    Err(e) => {
                                        toast.error(e.to_user_message(), ToastOptions::default());
                                        return;
                                    }
                                };

                                if let Ok(deck) = client().get_deck(deck_id, &session).await {
                                    let mb: Vec<DeckEntry> = deck.entries
                                        .iter()
                                        .filter(|e| e.deck_card.board.is_maybeboard())
                                        .cloned()
                                        .collect();
                                    mb_entries.set(mb);
                                    mb_stack.rewind();
                                    let current = *filter_reset_counter.peek();
                                    filter_reset_counter.set(current + 1);
                                }
                            });
                            toast.info(
                                "Maybeboard refreshed".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        } else {
                            // Search mode: re-fetch from API
                            let mut builder = filter_builder.peek().clone();
                            builder.set_is_token(false);
                            builder.set_limit(pagination_limit);
                            builder.set_offset(0);

                            let Ok(filter) = builder.build() else {
                                toast.warning(
                                    "Filter is empty".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                                return;
                            };

                            let session_signal = session;
                            let Some(session) = session.peek().clone() else {
                                toast.error("Session expired".to_string(), ToastOptions::default());
                                return;
                            };

                            stack.reset();
                            reset_image_ease();
                            last_search_filter.set(None);
                            current_offset.set(0);
                            pagination_exhausted.set(false);
                            is_loading_cards.set(true);

                            let filter_snapshot = filter_builder.peek().clone();

                            usage_buffer().record_search();
                            spawn(async move {
                                // Flush buffered usage (signals feed synergy
                                // ordering) before the refetch. Skips post
                                // directly at swipe time.
                                flush_once(&usage_buffer(), &client, &session_signal).await;
                                match client().search_deck_cards(deck_id, &filter, &session).await {
                                    Ok((cards_from_search, warming)) => {
                                        synergy_warming.set(warming);
                                        let deck_ids = deck_cards_ids();
                                        stack.replace(
                                            cards_from_search
                                                .into_iter()
                                                .filter(|card| {
                                                    card.scryfall_data.primary_image_url(ImageSize::Large).is_some()
                                                        && !card.scryfall_data.oracle_id.is_some_and(|oid| deck_ids.contains(&oid))
                                                })
                                                .collect(),
                                        );
                                        last_search_filter.set(Some(filter_snapshot));
                                        current_offset.set(pagination_limit);
                                        is_loading_cards.set(false);
                                    }
                                    Err(e) => {
                                        tracing::warn!("card search failed: {e}");
                                        toast.error(e.to_user_message(), ToastOptions::default());
                                        is_loading_cards.set(false);
                                    }
                                }
                            });

                            toast.info(
                                "Stack refreshed".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                    },
                    "Refresh"
                }
                {
                    let has_card = if add_source() == AddSource::Maybeboard {
                        mb_current_card().is_some()
                    } else {
                        current_card().is_some()
                    };
                    rsx! {
                        if has_card {
                            RulesButton { open: show_rules }
                        }
                    }
                }
            }

            HintDialog {
                open: swipe_hint_open,
                title: "Swipe to build",
                HintBullets {
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-success", "right" }
                        " to add a card to your deck."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-error", "left" }
                        " to skip it."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-warning", "up" }
                        " to send it to your maybeboard."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--accent-tertiary", "down" }
                        " to undo your last swipe."
                    }
                    HintBullet {
                        HintColored { color: "--accent-primary", "Synergy" }
                        " on keeps the stack to cards that work with your commander; turn it off to browse every legal card."
                    }
                }
                HintLine { "Sorting reorders whichever set you're viewing. It never changes which cards show. Filter or sort anytime." }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
                validate_before_apply: true,
                on_clear: move |_| clear_filters(),
            }
            }
        }
    }
}
