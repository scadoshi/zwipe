use super::components::card_info::{CardInfoDisplay, CardSkeleton};
use super::components::keyword_hint::{KeywordHintDialog, card_has_keywords};
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
            telemetry::usage_buffer::UsageBuffer,
        },
        screens::deck::card::{
            components::action_history::{
                CARDS_WARNING_THRESHOLD, MAX_CARDS_IN_STACK, SwipeAction,
            },
            filter::card_filter_sheet::CardFilterSheet,
        },
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::{get_deck::ClientGetDeck, search_deck_cards::ClientSearchDeckCards},
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
            card_filter::builder::CardFilterBuilder,
            filter_cards::{FilterCards, SortCards},
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

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();
    let mut last_search_filter: Signal<Option<CardFilterBuilder>> = use_context();
    let is_first_run = use_hook(|| std::cell::Cell::new(true));

    // Swipe vocabulary hint: auto-opens on this user's first visit, the
    // grayed "?" in the util bar reopens it on demand.
    let swipe_hint_open = use_one_time_hint(HINT_ADD_DECK_CARDS);
    let mut keyword_hint_open = use_signal(|| false);

    // When Some, the SwipeStack plays a keyframe entering from this direction
    // on the next top card, and clears it on animationend. Set by undo.
    let mut entering_direction: Signal<Option<Direction>> = use_signal(|| None);

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
    let mut mb_displayed_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut mb_current_index: Signal<usize> = use_signal(|| 0);
    let mut mb_action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);
    let mut mb_entering_direction: Signal<Option<Direction>> = use_signal(|| None);

    // Undo action history
    let mut action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);

    // Filters overlay at bottom of screen state
    let mut filters_overlay_open = use_signal(|| false);

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

    // Card iteration state
    let mut current_index = use_signal(|| 0_usize);

    // Pagination state
    let mut current_offset = use_signal(|| 0_u32);
    let mut is_loading_more = use_signal(|| false);
    let pagination_limit = 25_u32; // Matches backend default
    // Keep the buffer comfortably ahead of the STACK_DEPTH window so the
    // stack never visibly shrinks while a batch is in flight.
    let load_more_threshold = 5_usize;
    let mut pagination_exhausted = use_signal(|| false);

    let mut is_loading_cards = use_signal(|| false);

    let current_card = move || {
        let idx = current_index();
        cards().get(idx).cloned()
    };

    // Load more cards with pagination and de-duplication
    let mut load_more_cards = move || {
        // Check if we've hit the card limit
        let current_card_count = cards().len();
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
                    let existing_cards = cards();
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
                        let mut updated_cards = existing_cards;
                        updated_cards.extend(unique_new_cards);
                        cards.set(updated_cards);

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
    let mut advance_after_commit = move || {
        let total = cards().len();
        if current_index() + 1 < total {
            current_index.set(current_index() + 1);
            if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total)
                && total.is_multiple_of(100)
            {
                toast.info(
                    "Approaching card limit, consider refreshing".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(2000)),
                );
            }
            // Trigger a pagination prefetch when we're within the threshold.
            if total > 0 && current_index() + 1 >= total.saturating_sub(load_more_threshold) {
                load_more_cards();
            }
        } else {
            // At the end — try to load more, else inform the user.
            if pagination_exhausted() {
                toast.warning("End of results".to_string(), ToastOptions::default());
            } else {
                load_more_cards();
            }
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
                Ok(_) => {
                    if let Some(oid) = oracle_id {
                        deck_cards_ids.write().insert(oid);
                    }
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
        let Some(action) = action_history.write().pop() else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        // Can't undo if we're at the first card
        if current_index() == 0 {
            toast.warning("No previous card".to_string(), ToastOptions::default());
            action_history.write().push(action); // Put it back
            return;
        }

        // Go back one card — the previously-swiped card becomes the new top.
        current_index.set(current_index() - 1);
        // Ask the stack to play the enter animation from the direction the
        // card originally exited.
        entering_direction.set(Some(action.exited().clone()));

        match action {
            // Remove-screen-only variant; never pushed on this screen.
            SwipeAction::MoveBoard { .. } => {}
            SwipeAction::Skip { .. } => {
                // Just showing previous card - done!
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do { ref card, .. } => {
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
                            action_history.write().push(action); // Restore history
                            current_index.set(current_index() + 1); // Restore index
                            entering_direction.set(None);
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
            SwipeAction::Maybeboard { ref card, .. } => {
                let card_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            action_history.write().push(action);
                            current_index.set(current_index() + 1);
                            entering_direction.set(None);
                            return;
                        }
                    };

                    match client().delete_deck_card(deck_id, card_id, &session).await {
                        Ok(_) => {
                            if let Some(oid) = oracle_id {
                                deck_cards_ids.write().remove(&oid);
                            }
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
        if add_source() == AddSource::Maybeboard {
            // Maybeboard mode: "cleared" means truly blank
            if filter_builder.read().is_empty() {
                toast.warning("Filter already cleared".to_string(), opts);
            } else {
                filter_builder.write().clear();
                let current = *filter_reset_counter.peek();
                filter_reset_counter.set(current + 1);
                toast.info("Filter cleared".to_string(), opts);
            }
        } else {
            // Search mode: "cleared" means only deck-context defaults
            if filter_builder.read().is_empty_ignoring_deck_context() {
                toast.warning("Filter already cleared".to_string(), opts);
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
                cards.set(vec![]);
                current_index.set(0);
                toast.info("Filter cleared".to_string(), opts);
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
                    if cards.peek().is_empty() {
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
            // Preserve cards if the filter hasn't changed since last search.
            let filter_unchanged = last_search_filter
                .peek()
                .as_ref()
                .map(|prev| {
                    let mut prev_b = prev.clone();
                    prev_b.set_is_token(false);
                    prev_b.set_limit(pagination_limit);
                    prev_b.set_offset(0);
                    prev_b == builder
                })
                .unwrap_or(false);

            if filter_unchanged && !cards.peek().is_empty() {
                // Restore pagination offset so load-more picks up from the right place.
                current_offset.set(cards.peek().len() as u32);
                return;
            }
        }

        // ── Clear and re-fetch ────────────────────────────────────────
        // Reaches here when:
        //   - explicit user action (refresh / apply filter), OR
        //   - initial mount with a different/new filter, OR
        //   - initial mount with no existing cards
        cards.set(vec![]);
        last_search_filter.set(None);
        current_offset.set(0);
        current_index.set(0);
        pagination_exhausted.set(false);

        // Gate (Search mode, once the deck is loaded): only auto-serve a
        // meaningful query. A commander format *with* a commander pulls
        // synergy suggestions from cache; everything else needs a real user
        // filter. Otherwise leave the stack empty and nudge the user to filter
        // — an empty screen with no prompt reads as "no cards exist".
        if *deck_loaded.peek() && matches!(*add_source.peek(), AddSource::Search) {
            let is_commander_format = deck_format.peek().is_some_and(|f| f.has_commander());
            let has_commander = *deck_has_commander.peek();
            let has_user_filter = !filter_builder.peek().is_empty_ignoring_deck_context();
            let serve = if is_commander_format {
                has_commander || has_user_filter
            } else {
                has_user_filter
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
        let Some(session) = session.peek().clone() else {
            toast.error("Session expired".to_string(), ToastOptions::default());
            return;
        };

        is_loading_cards.set(true);

        // Snapshot the filter builder state before the async block owns context.
        let filter_snapshot = filter_builder.peek().clone();

        usage_buffer().record_search();
        spawn(async move {
            match client().search_deck_cards(deck_id, &filter, &session).await {
                Ok((cards_from_search, warming)) => {
                    synergy_warming.set(warming);
                    let deck_ids = deck_cards_ids();
                    cards.set(
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
        let mut filtered = if builder.is_empty() {
            cards
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => cards.filter_by(&filter),
                Err(_) => cards,
            }
        };
        filtered.sort_by_filter(&builder);
        mb_displayed_cards.set(filtered);
        mb_current_index.set(0);
    });

    let mb_current_card = move || {
        let cards = mb_displayed_cards();
        if cards.is_empty() {
            return None;
        }
        let idx = mb_current_index() % cards.len();
        cards.get(idx).cloned()
    };

    let mut mb_promote_to_deck = move |card: Card| {
        let scryfall_data_id = card.scryfall_data.id;
        let request = HttpUpdateDeckCard::new(None, Some("deck".to_string()));

        // Optimistic: remove from maybeboard lists
        mb_entries
            .write()
            .retain(|e| e.card.scryfall_data.id != scryfall_data_id);
        let idx = mb_current_index();
        if idx < mb_displayed_cards.read().len() {
            mb_displayed_cards.write().remove(idx);
        }
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
        let Some(action) = mb_action_history.write().pop() else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        mb_entering_direction.set(Some(action.exited().clone()));

        match action {
            SwipeAction::Skip { .. } => {
                let len = mb_displayed_cards().len();
                if len == 0 {
                    mb_entering_direction.set(None);
                    return;
                }
                let idx = mb_current_index();
                let prev = if idx == 0 { len - 1 } else { idx - 1 };
                mb_current_index.set(prev);
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do { card, .. } => {
                let card = *card;
                let idx = mb_current_index();
                mb_displayed_cards.write().insert(idx, card.clone());

                let scryfall_data_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;
                let request = HttpUpdateDeckCard::new(None, Some("maybeboard".to_string()));

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            mb_entering_direction.set(None);
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
            SwipeAction::Maybeboard { .. } | SwipeAction::MoveBoard { .. } => {
                // Shouldn't happen in maybeboard mode, but handle gracefully
                toast.info("Nothing to undo".to_string(), ToastOptions::default());
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
                                    mb_current_index.set(0);
                                    mb_action_history.write().clear();
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
                        if !cards().is_empty() {
                            SwipeStack {
                                cards: {
                                    let all = cards();
                                    all.into_iter().skip(current_index()).take(crate::inbound::components::interactions::swipe::STACK_DEPTH).collect::<Vec<_>>()
                                },
                                config: swipe_config,
                                entering: entering_direction,
                                on_swipe_left: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Left);
                                    usage_buffer().record_signal(commander_oracle_id(), card.scryfall_data.oracle_id, Direction::Left);
                                    action_history.write().push(SwipeAction::Skip { card: Box::new(card), exited: Direction::Left });
                                    toast.info("Skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    advance_after_commit();
                                },
                                on_swipe_right: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Right);
                                    usage_buffer().record_signal(commander_oracle_id(), card.scryfall_data.oracle_id, Direction::Right);
                                    action_history.write().push(SwipeAction::Do { card: Box::new(card.clone()), exited: Direction::Right });
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
                                    action_history.write().push(SwipeAction::Maybeboard { card: Box::new(card.clone()), exited: Direction::Up });
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
                                CardInfoDisplay { card }
                            }
                        } else if is_loading_cards() {
                            CardSkeleton { is_loading: true }
                        } else {
                            CardSkeleton {}
                        }
                    } else {
                        // Maybeboard mode
                        if !mb_displayed_cards().is_empty() {
                            SwipeStack {
                                cards: {
                                    let all = mb_displayed_cards();
                                    all.into_iter().skip(mb_current_index()).take(crate::inbound::components::interactions::swipe::STACK_DEPTH).collect::<Vec<_>>()
                                },
                                config: swipe_config,
                                entering: mb_entering_direction,
                                on_swipe_left: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Left);
                                    mb_action_history.write().push(SwipeAction::Skip { card: Box::new(card), exited: Direction::Left });
                                    toast.info("Skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    let len = mb_displayed_cards().len();
                                    if len > 0 {
                                        mb_current_index.set((mb_current_index() + 1) % len);
                                    }
                                },
                                on_swipe_right: move |card: Card| {
                                    usage_buffer().record_swipe(Direction::Right);
                                    mb_action_history.write().push(SwipeAction::Do { card: Box::new(card.clone()), exited: Direction::Right });
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
                                CardInfoDisplay { card }
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
                    if !filter_builder.read().is_empty_ignoring_deck_context() {
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
                                    mb_current_index.set(0);
                                    mb_action_history.write().clear();
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
                                    "filter is empty".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                                return;
                            };

                            let Some(session) = session.peek().clone() else {
                                toast.error("Session expired".to_string(), ToastOptions::default());
                                return;
                            };

                            cards.set(vec![]);
                            last_search_filter.set(None);
                            current_offset.set(0);
                            current_index.set(0);
                            pagination_exhausted.set(false);
                            is_loading_cards.set(true);

                            let filter_snapshot = filter_builder.peek().clone();

                            usage_buffer().record_search();
                            spawn(async move {
                                match client().search_deck_cards(deck_id, &filter, &session).await {
                                    Ok((cards_from_search, warming)) => {
                                        synergy_warming.set(warming);
                                        let deck_ids = deck_cards_ids();
                                        cards.set(
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
                    let kw_card = if add_source() == AddSource::Maybeboard {
                        mb_current_card()
                    } else {
                        current_card()
                    };
                    rsx! {
                        if kw_card.as_ref().is_some_and(card_has_keywords) {
                            button {
                                class: "util-btn",
                                onclick: move |_| keyword_hint_open.set(true),
                                "Keywords"
                            }
                        }
                    }
                }
            }

            {
                let kw_card = if add_source() == AddSource::Maybeboard {
                    mb_current_card()
                } else {
                    current_card()
                };
                rsx! {
                    if let Some(card) = kw_card {
                        KeywordHintDialog { open: keyword_hint_open, card }
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
