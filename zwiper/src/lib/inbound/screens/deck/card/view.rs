use std::collections::{HashMap, HashSet};

use components::icons::{self};
use components::{
    Button, ButtonVariant, CardGroup, CardsViewSwitch, Chip, DeckEditActions, DeckRowHost,
    DeckSection, DeckStackDrawer, DeckStacks, DockAction, DockFilter, GroupBuyDialog, StackDetails,
    TailPlace, Toasts, group_tail, use_group_buy, use_settle, use_stack_details_sync, use_toast,
};
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus::router::Navigator;
use tokio::time;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{self, Board, Deck};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_ui::deck::command_zone::Slot;
use zwipe_ui::deck_cards::keys::RowInfo;
use zwipe_ui::deck_cards::stacks::{
    CardsView, DEAL_DURATION, DEAL_STAGGER_CAP, DealCue, Reveal, deal_delay,
};
use zwipe_ui::deck_cards::{FeaturedOpen, Write, Zone, featured_open};
use zwipe_ui::filter;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_cards::{self, Action, Destination, Effect, Op, StackGroup, State};
use zwipe_ui::stores::undo::UndoAction as SharedUndo;

use super::components::card_row::{CardRow, OtagDescribe, OtagExamplesOpen, ShowRowArt};
use super::components::filter_store::{FilterScope, FilterStore};
use super::components::image_preview::ImagePreview;
use super::components::printing_sheet::PrintingSheet;
use super::components::quick_add::QuickAdd;
use super::components::undo_log::{UndoAction, UndoLog, UndoStore, take_newest_in};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::{self, OneTimeHintDialog};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen, component, screen};
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::filter::card_filter_sheet::{
    CardFilterSheet, CollapseExpanded,
};
use crate::inbound::screens::deck::card::filter::deck_cards::DeckCards;
use crate::inbound::screens::deck::components::featured_cards::FeaturedCards;
use crate::inbound::screens::deck::components::skeletons::{
    ChipGhost, ChipRowGhost, DeckCardListSkeleton, QuickAddSkeleton,
};
use crate::inbound::screens::deck::missing_deck;
use crate::inbound::screens::deck::view::HeldDeck;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;
use crate::outbound::cards_view_store;

/// The app's copy of a shared undo entry, for the screen's live log.
fn to_app_undo(action: SharedUndo) -> UndoAction {
    match action {
        SharedUndo::Added { card_id, card_name } => UndoAction::Added { card_id, card_name },
        SharedUndo::Removed { entry, baseline } => UndoAction::Removed {
            entry: *entry,
            baseline,
        },
        SharedUndo::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        } => UndoAction::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        },
        SharedUndo::MovedBoard {
            card_id,
            card_name,
            from,
        } => UndoAction::MovedBoard {
            card_id,
            card_name,
            from,
        },
        SharedUndo::MvpToggled {
            card_id,
            card_name,
            before,
        } => UndoAction::MvpToggled {
            card_id,
            card_name,
            before,
        },
        SharedUndo::PrintingChanged { old_card, new_id } => UndoAction::PrintingChanged {
            old_card: *old_card,
            new_id,
        },
        SharedUndo::CommandZonePrintingChanged {
            slot,
            old_card,
            new_id,
        } => UndoAction::CommandZonePrintingChanged {
            slot,
            old_card: *old_card,
            new_id,
        },
    }
}

/// The shared copy of an app undo entry, for the model's Undo.
fn to_shared_undo(action: UndoAction) -> SharedUndo {
    match action {
        UndoAction::Added { card_id, card_name } => SharedUndo::Added { card_id, card_name },
        UndoAction::Removed { entry, baseline } => SharedUndo::Removed {
            entry: Box::new(entry),
            baseline,
        },
        UndoAction::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        } => SharedUndo::QuantityChanged {
            card_id,
            card_name,
            net,
            baseline,
        },
        UndoAction::MovedBoard {
            card_id,
            card_name,
            from,
        } => SharedUndo::MovedBoard {
            card_id,
            card_name,
            from,
        },
        UndoAction::MvpToggled {
            card_id,
            card_name,
            before,
        } => SharedUndo::MvpToggled {
            card_id,
            card_name,
            before,
        },
        UndoAction::PrintingChanged { old_card, new_id } => SharedUndo::PrintingChanged {
            old_card: Box::new(old_card),
            new_id,
        },
        UndoAction::CommandZonePrintingChanged {
            slot,
            old_card,
            new_id,
        } => SharedUndo::CommandZonePrintingChanged {
            slot,
            old_card: Box::new(old_card),
            new_id,
        },
    }
}

/// The deck cards view's model with the handles that perform its effects.
/// Copy, so every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    undo_log: UndoLog,
    filter: Signal<CardQueryBuilder>,
    counter: Signal<u32>,
    expanded_card: Signal<Option<Uuid>>,
    printing_sheet_open: Signal<bool>,
    hint_open: Signal<bool>,
    /// The deck the deck route already holds, which the screen opens on without a fetch.
    held: Option<HeldDeck>,
    /// The stacks' reveal and open card, which a printing change rekeys.
    reveal: Signal<Reveal>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        // A star or an undo can add or take a featured card and move the cards under it, so the
        // page settles around it.
        if matches!(action, Action::ToggleMvp { .. } | Action::Undo(_)) {
            components::settle::watch();
        }
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let Self {
            navigator,
            toast,
            authed,
            session,
            client,
            usage_buffer,
            undo_log,
            mut filter,
            mut counter,
            mut expanded_card,
            mut printing_sheet_open,
            hint_open,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            Effect::FetchDeck { ticket, deck_id } => self.fetch_deck(ticket, deck_id),
            effect @ (Effect::FetchTokens { .. }
            | Effect::SearchCards { .. }
            | Effect::FetchPrintings { .. }) => {
                self.fetch(effect);
            }
            effect @ (Effect::CreateCard { .. }
            | Effect::PatchCard { .. }
            | Effect::DeleteCard { .. }
            | Effect::PatchProfile { .. }) => self.request(effect),
            Effect::Debounce { ticket, wait } => {
                spawn(async move {
                    time::sleep(wait).await;
                    self.dispatch(Action::Debounced { ticket });
                });
            }
            Effect::Flush {
                deck_id,
                card_id,
                write,
            } => flush(authed, deck_id, card_id, write),
            Effect::RecordRemoval { deck_id, oracle_id } => {
                usage_buffer.peek().record_removal(deck_id, oracle_id);
            }
            Effect::RecordUndo(action) => undo_log.push(to_app_undo(*action)),
            Effect::PrintingSwapped { old_id, new_id } => {
                let mut reveal = self.reveal;
                reveal.write().rekey(old_id, new_id);
            }
            Effect::ForgetRemoval { card_id } => {
                let mut log = undo_log.0;
                take_newest_in(
                    &mut log.write(),
                    |a| matches!(a, UndoAction::Removed { entry, .. } if entry.card.scryfall_data.id == card_id),
                );
            }
            Effect::ClearFilter => {
                filter.write().clear();
            }
            Effect::Refilter => {
                let current = *counter.peek();
                counter.set(current.wrapping_add(1));
            }
            Effect::Collapse => expanded_card.set(None),
            Effect::OpenPrintings => printing_sheet_open.set(true),
            Effect::Navigate(destination) => match route(destination) {
                Some(route) => {
                    navigator.push(route);
                }
                None => navigator.go_back(),
            },
            Effect::Replace(destination) => match route(destination) {
                Some(route) => {
                    navigator.replace(route);
                }
                None => navigator.go_back(),
            },
            Effect::Toast(t) => ui_toast::raise(toast, t),
        }
    }

    /// Sends one quiet fetch and feeds its reply back through
    /// [`Self::dispatch`].
    fn fetch(self, effect: Effect) {
        let Self {
            authed,
            usage_buffer,
            ..
        } = self;
        match effect {
            Effect::FetchTokens { ticket, deck_id } => {
                spawn(async move {
                    // Quiet: tokens are a decorative section, so a failure
                    // reads as "no tokens"; the facade records it.
                    let tokens = authed
                        .run_quiet("load_tokens", |c, s| async move {
                            c.fetch_deck_tokens(deck_id, &s).await
                        })
                        .await
                        .unwrap_or_default();
                    self.dispatch(Action::TokensLoaded { ticket, tokens });
                });
            }
            Effect::SearchCards {
                ticket,
                deck_id,
                query,
            } => {
                usage_buffer.peek().record_search();
                spawn(async move {
                    // Quiet: fires per debounced keystroke; the dropdown just closes.
                    let page = authed
                        .run_quiet_at(
                            component::QUICK_ADD,
                            "quick_add_search",
                            |c, s| async move { c.search_deck_cards(deck_id, &query, &s).await },
                        )
                        .await;
                    self.dispatch(match page {
                        Some(page) => Action::Searched {
                            ticket,
                            cards: page.cards,
                        },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::FetchPrintings { ticket, oracle_id } => {
                let client = self.client;
                spawn(async move {
                    // Quiet: a failure adds the picked printing as before.
                    let http = client.peek().clone();
                    self.dispatch(match http.fetch_printings(oracle_id).await {
                        Ok(printings) => Action::PrintingsLoaded { ticket, printings },
                        Err(e) => {
                            tracing::warn!("quick add printings failed: {e}");
                            Action::Failed { ticket }
                        }
                    });
                });
            }
            _ => {}
        }
    }

    /// Sends one write and feeds its reply back through [`Self::dispatch`].
    /// Each reports and toasts its own failure.
    fn request(self, effect: Effect) {
        let authed = self.authed;
        match effect {
            Effect::CreateCard {
                ticket,
                deck_id,
                request,
                op,
            } => {
                spawn(async move {
                    let call = |c: ZwipeClient, s: Session| async move {
                        c.create_deck_card(deck_id, &request, &s).await
                    };
                    let created = if op == Op::QuickAddCard {
                        authed.run_at(component::QUICK_ADD, op.name(), call).await
                    } else {
                        authed.try_run(op.name(), call).await.ok()
                    };
                    self.dispatch(match created {
                        Some(deck_card) => Action::Created {
                            ticket,
                            deck_card: Box::new(deck_card),
                        },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::PatchCard {
                ticket,
                deck_id,
                card_id,
                request,
                op,
            } => {
                spawn(async move {
                    let patched = authed
                        .try_run(op.name(), |c, s| async move {
                            c.update_deck_card(deck_id, card_id, &request, &s).await
                        })
                        .await;
                    self.dispatch(match patched {
                        Ok(deck_card) => Action::Patched {
                            ticket,
                            deck_card: Box::new(deck_card),
                        },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::DeleteCard {
                ticket,
                deck_id,
                card_id,
                op,
            } => {
                spawn(async move {
                    let deleted = authed
                        .try_run(op.name(), |c, s| async move {
                            // An undone add goes back without a removal
                            // suppression, so the card can serve in Add again.
                            if op == Op::Undo {
                                c.delete_undone_deck_card(deck_id, card_id, &s).await
                            } else {
                                c.delete_deck_card(deck_id, card_id, &s).await
                            }
                        })
                        .await;
                    self.dispatch(match deleted {
                        Ok(()) => Action::Deleted { ticket },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::PatchProfile {
                ticket,
                deck_id,
                request,
                op,
            } => {
                spawn(async move {
                    let patched = authed
                        .try_run(op.name(), |c, s| async move {
                            c.update_deck_profile(deck_id, &request, &s).await
                        })
                        .await;
                    self.dispatch(match patched {
                        Ok(_) => Action::ProfilePatched { ticket },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            _ => {}
        }
    }

    /// The whole load rides one facade call: only `fetch_deck`'s failure
    /// aborts (reported and toasted); the profile and command zone fetches
    /// keep their tolerant handling inside. The profile goes to the model as
    /// soon as it lands, so the header draws before the cards.
    fn fetch_deck(self, ticket: Ticket, deck_id: Uuid) {
        // The deck the route holds answers at once, with no fetch and no ghost.
        let held = self
            .held
            .and_then(|held| held.0.peek().clone())
            .filter(|deck| deck.deck_profile.id == deck_id);
        if let Some(deck) = held {
            let zone = Zone::of_deck(&deck);
            self.dispatch(Action::ProfileLoaded {
                ticket,
                profile: Box::new(deck.deck_profile.clone()),
            });
            self.dispatch(Action::Loaded {
                ticket,
                deck: Box::new(deck),
                zone: Box::new(zone),
            });
            return;
        }
        let Self { authed, .. } = self;
        spawn(async move {
            let loaded = authed
                .try_run("load_deck", |c, s| async move {
                    let deck = c.fetch_deck(deck_id, &s).await?;
                    let mut zone = Zone::default();
                    if let Ok(profile) = c.fetch_deck_profile(deck_id, &s).await {
                        let requests = deck_cards::zone_requests(&profile);
                        self.dispatch(Action::ProfileLoaded {
                            ticket,
                            profile: Box::new(profile),
                        });
                        for (slot, card_id) in requests {
                            *zone.slot_mut(slot) = c.fetch_card(card_id).await.ok();
                        }
                    }
                    Ok((deck, zone))
                })
                .await;
            self.dispatch(missing_deck::settle(
                loaded,
                |(deck, zone)| Action::Loaded {
                    ticket,
                    deck: Box::new(deck),
                    zone: Box::new(zone),
                },
                Action::Gone { ticket },
                Action::Failed { ticket },
            ));
        });
    }
}

/// The screen's model loaded on `deck`, as a load answered by it would leave it, with the effects
/// that load still asks for (the tokens, the hint).
fn seeded_state(deck_id: Uuid, deck: &Deck, filter: Box<CardQueryBuilder>) -> (State, Vec<Effect>) {
    let mut state = State::new();
    let mut pending = Vec::new();
    for effect in state.update(Action::Load { deck_id, filter }) {
        let Effect::FetchDeck { ticket, .. } = effect else {
            pending.push(effect);
            continue;
        };
        let zone = Zone::of_deck(deck);
        pending.extend(state.update(Action::ProfileLoaded {
            ticket,
            profile: Box::new(deck.deck_profile.clone()),
        }));
        pending.extend(state.update(Action::Loaded {
            ticket,
            deck: Box::new(deck.clone()),
            zone: Box::new(zone),
        }));
    }
    (state, pending)
}

/// The route a destination opens, or `None` for the screen before this one.
fn route(destination: Destination) -> Option<Router> {
    match destination {
        Destination::Previous => None,
        Destination::AddCards(deck_id) => Some(Router::AddDeckCard { deck_id }),
        Destination::RemoveCards(deck_id) => Some(Router::RemoveDeckCard { deck_id }),
        Destination::DeckList => Some(Router::DeckList {}),
    }
}

/// Posts a burst the screen leaves behind. Detached, since the screen's scope
/// is going away, so a failure only reports and toasts.
fn flush(authed: Authed, deck_id: Uuid, card_id: Uuid, write: Write) {
    let target = match write {
        Write::Nothing => return,
        Write::Delete => None,
        Write::Set(target) => Some(target),
    };
    spawn_forever(async move {
        let _ = authed
            .run("flush_quantity", |c, s| async move {
                if let Some(target) = target {
                    // PATCH: the burst's absolute target, replay-safe.
                    let request = HttpPatchDeckCard::new(Some(target), None);
                    c.update_deck_card(deck_id, card_id, &request, &s)
                        .await
                        .map(|_| ())
                } else {
                    c.delete_deck_card(deck_id, card_id, &s).await
                }
            })
            .await;
    });
}

#[component]
pub fn View(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    // This screen's own filter: seeded from the per-(screen, deck) store,
    // provided as context so the filter modules bind to it, parked on leave.
    let filter_store: FilterStore = use_context();
    let filter_builder: Signal<CardQueryBuilder> = use_signal(|| {
        filter_store
            .restore(FilterScope::Cards, deck_id)
            .unwrap_or_default()
    });
    use_context_provider(|| filter_builder);
    use_drop(move || {
        let mut store = filter_store;
        store.park(FilterScope::Cards, deck_id, filter_builder.peek().clone());
    });

    let mut filters_overlay_open = use_signal(|| false);

    // The filter sheet bumps this on Apply, and the model asks for a bump
    // whenever the deck changes; the sheet's sections clear their typed
    // searches on it. Readers only see that it moved, so bumps wrap.
    let filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();
    let authed = use_authed(Screen::Deck(DeckScreen::CardView));

    // Undo log: the model records entries and the Undo button pops them.
    // Seeded from the app-level per-deck store on mount and parked back on
    // leave, so the stack survives navigating around the app.
    let undo_store: UndoStore = use_context();
    let undo_log = UndoLog(use_signal(|| undo_store.restore(deck_id)));
    use_drop(move || {
        let mut store = undo_store;
        store.park(deck_id, undo_log.0.peek().clone());
    });

    // The deck's cards as they loaded, for the filter sheet's value pickers.
    let mut deck_cards_for_filter: Signal<Vec<Card>> = use_signal(Vec::new);
    use_context_provider(|| DeckCards(deck_cards_for_filter));

    // Shared with every CardRow so compact-row prices use the deck's currency.
    let mut price_budget_currency: Signal<PriceCurrency> = use_signal(|| PriceCurrency::Usd);
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
    // the controls toggles it.
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    // Which card row is expanded (None = all collapsed)
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);

    // Card image preview: stores the card to preview (None = closed). Carries the
    // full ScryfallData so the modal's FlippableCardImage can read both faces of DFCs.
    let preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let printing_sheet_open: Signal<bool> = use_signal(|| false);
    // Controls the dismiss animation before clearing the URL
    let preview_dismissing: Signal<bool> = use_signal(|| false);
    // Set by the filter sheet's Apply: the next counter bump collapses the
    // expanded card.
    let mut should_collapse_expanded: Signal<bool> = use_signal(|| false);
    use_context_provider(|| CollapseExpanded(should_collapse_expanded));

    let deck_cards_hint_open = use_signal(|| false);
    // The stacks' reveal and open card, before the driver, which rekeys it when a printing changes.
    let reveal = use_signal(Reveal::default);

    // With the deck the route holds, the screen starts loaded on it, so it never draws its ghost;
    // what that load asks for next runs once it mounts.
    let held = try_use_context::<HeldDeck>();
    let seeded = use_hook(|| {
        let deck = held
            .and_then(|held| held.0.peek().clone())
            .filter(|deck| deck.deck_profile.id == deck_id)?;
        let filter = Box::new(filter_builder.peek().clone());
        Some(seeded_state(deck_id, &deck, filter))
    });
    let (start, pending) = seeded.map_or_else(
        || (State::new(), None),
        |(state, effects)| (state, Some(effects)),
    );
    let driver = Driver {
        model: use_signal(move || start),
        navigator,
        toast,
        authed,
        session,
        client,
        usage_buffer,
        undo_log,
        filter: filter_builder,
        counter: filter_reset_counter,
        expanded_card,
        printing_sheet_open,
        hint_open: deck_cards_hint_open,
        held,
        reveal,
    };

    use_effect(move || {
        let filter = Box::new(filter_builder.peek().clone());
        match pending.clone() {
            Some(effects) => {
                for effect in effects {
                    driver.perform(effect);
                }
            }
            None => driver.dispatch(Action::Load { deck_id, filter }),
        }
    });

    // A tap inside the debounce window must still reach the server when the
    // user leaves the screen: every waiting burst posts on unmount.
    use_drop(move || {
        let mut model = driver.model;
        let effects = model.write().update(Action::Leave);
        for effect in effects {
            driver.perform(effect);
        }
    });

    // Each counter bump hands the filter as it stands to the model, which
    // groups the main board by it.
    use_effect(move || {
        let _ = filter_reset_counter();
        let filter = filter_builder.peek().clone();
        driver.dispatch(Action::Filter(Box::new(filter)));
        if *should_collapse_expanded.peek() {
            let mut expanded_card = expanded_card;
            expanded_card.set(None);
            should_collapse_expanded.set(false);
        }
    });

    // The filter sheet's pickers and the rows' prices read the model through
    // their own context signals.
    use_effect(move || {
        let model = driver.model.read();
        if *deck_cards_for_filter.peek() != model.filter_cards() {
            deck_cards_for_filter.set(model.filter_cards().to_vec());
        }
        if *price_budget_currency.peek() != model.currency() {
            price_budget_currency.set(model.currency());
        }
    });

    // List or Stacks, as last left on this device.
    let view = use_signal(cards_view_store::restored);
    // The stacks' reveal and open card, the face each card shows, and the
    // folds as the model holds them, which the stack headers read.
    let details_open = use_signal(|| false);
    let stack_details = StackDetails {
        reveal,
        details_open,
    };
    let stack_faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    let mut folds: Signal<HashSet<String>> = use_signal(HashSet::new);
    use_effect(move || {
        let model = driver.model.read();
        if *folds.peek() != *model.collapsed() {
            folds.set(model.collapsed().clone());
        }
    });
    let listed_ids = use_memo(move || {
        let model = driver.model.read();
        let live = filter_builder.read();
        let display = model.display(&live);
        model
            .stack_groups(&display, &live)
            .into_iter()
            .flat_map(|group| group.rows)
            .map(|row| row.card.scryfall_data.id)
            .collect::<Vec<_>>()
    });
    use_stack_details_sync(reveal, details_open, listed_ids);
    // The stacks deal in on entering Stacks, once the deck is in, for as
    // long as the longest stack takes.
    let mut deal_cue = use_signal(DealCue::default);
    let mut dealing = use_signal(|| false);
    use_effect(move || {
        let shown = view();
        let ready = driver.model.read().is_loaded();
        if deal_cue.write().on(shown, ready) {
            dealing.set(true);
            spawn(async move {
                time::sleep(deal_delay(DEAL_STAGGER_CAP).saturating_add(DEAL_DURATION)).await;
                dealing.set(false);
            });
        }
    });
    let group_buy = use_group_buy();
    // A reveal in the stacks pushes the stacks below it down, and they slide
    // there and back instead of snapping.
    use_settle();

    let filter = filter_builder.read().clone();
    let model = driver.model.read();
    let loaded = model.is_loaded();
    let deck_name = model.profile().map_or_default(|p| p.name.to_string());
    let deck_format = model.profile().and_then(|p| p.format);
    let deck_power = model.profile().and_then(|p| p.power_level);
    let deck_tags = model.profile().map_or_default(|p| p.tags.clone());
    let deck_other_tags = model.profile().map_or_default(|p| p.other_tags.clone());
    let featured_cards: Vec<(Card, String)> = model
        .featured()
        .into_iter()
        .map(|(card, role)| (card, role.to_string()))
        .collect();
    let display = model.display(&filter);
    let group_by = model.group_by();
    let show_deck = model.board_on(Board::Deck);
    let show_maybe = model.board_on(Board::Maybeboard);
    let show_side = model.board_on(Board::Sideboard);
    let maybe_enabled = model.board_enabled(Board::Maybeboard);
    let side_enabled = model.board_enabled(Board::Sideboard);
    let show_tokens = model.show_tokens();
    let show_command_zone = model.show_command_zone();
    let offers_command_zone = model.offers_command_zone();
    let tokens = model.tokens(&filter);
    let expanded = |key: &str| !model.is_collapsed(key);
    let tokens_expanded = expanded(deck_cards::TOKENS_KEY);
    let maybe_expanded = expanded(deck_cards::MAYBEBOARD_KEY);
    let side_expanded = expanded(deck_cards::SIDEBOARD_KEY);
    let lands_expanded = expanded(deck_cards::LANDS_KEY);
    let commander_expanded = expanded(Slot::Commander.group_key());
    let background_expanded = expanded(Slot::Background.group_key());
    let spell_expanded = expanded(Slot::SignatureSpell.group_key());
    let zone_card = |slot: Slot| {
        display
            .zone
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, card)| card.clone())
    };
    let displayed_commander = zone_card(Slot::Commander);
    let displayed_partner = zone_card(Slot::Partner);
    let displayed_background = zone_card(Slot::Background);
    let displayed_signature_spell = zone_card(Slot::SignatureSpell);
    let has_partner = displayed_partner.is_some();
    // Each group with its header and its rows' quantity, board and star.
    let mut groups = Vec::with_capacity(display.groups.len());
    for group in &display.groups {
        let rows: Vec<_> = group
            .cards
            .iter()
            .map(|card| (card.clone(), model.row(card.scryfall_data.id)))
            .collect();
        groups.push((
            deck_cards::group_title(&group.label, model.copies(&group.cards)),
            group.key.clone(),
            group.pips.clone(),
            !model.is_collapsed(&group.key),
            rows,
        ));
    }
    // The fold-all button covers every section on screen.
    let mut section_keys: Vec<String> = display.groups.iter().map(|g| g.key.clone()).collect();
    section_keys.push(deck_cards::LANDS_KEY.to_owned());
    if show_tokens {
        section_keys.push(deck_cards::TOKENS_KEY.to_owned());
    }
    if show_maybe && !display.maybe.is_empty() {
        section_keys.push(deck_cards::MAYBEBOARD_KEY.to_owned());
    }
    if show_side && !display.side.is_empty() {
        section_keys.push(deck_cards::SIDEBOARD_KEY.to_owned());
    }
    if show_command_zone {
        for (slot, shown) in [
            (
                Slot::Commander,
                displayed_commander.is_some() || has_partner,
            ),
            (Slot::Background, displayed_background.is_some()),
            (Slot::SignatureSpell, displayed_signature_spell.is_some()),
        ] {
            if shown {
                section_keys.push(slot.group_key().to_owned());
            }
        }
    }
    let all_collapsed = model.all_collapsed(&section_keys);
    let lands_title = deck_cards::lands_title(model.copies(&display.lands));
    let lands: Vec<_> = display
        .lands
        .iter()
        .map(|card| (card.clone(), model.row(card.scryfall_data.id)))
        .collect();
    let no_cards = model.no_cards(&display);
    let printing_card = model.printing().cloned();
    let quick_query = model.quick_query().to_owned();
    let quick_results = model.quick_add();
    let stacks_on = view() == CardsView::Stacks;
    // The groups as stacks, which the List's group carts read too.
    let stack_groups = model.stack_groups(&display, &filter);
    let stack_boards: HashMap<Uuid, Board> = stack_groups
        .iter()
        .flat_map(|group| &group.rows)
        .filter_map(|row| {
            let id = row.card.scryfall_data.id;
            model.row(id).map(|r| (id, r.board))
        })
        .collect();
    drop(model);
    let stack_rows: Vec<RowInfo> = stack_groups
        .iter()
        .flat_map(|group| group.rows.iter().cloned())
        .collect();
    // Each List group's header and rows, by its fold key, for its cart.
    let group_rows: HashMap<String, (String, Vec<RowInfo>)> = stack_groups
        .iter()
        .map(|group| (group.key.clone(), (group.title.clone(), group.rows.clone())))
        .collect();
    let currency = price_budget_currency();
    // A List group's price and cart on the line under its title, as a stack's header shows
    // them: nothing for a group with neither, such as the tokens.
    let tail = |key: &str| -> Option<Element> {
        let (title, rows) = group_rows.get(key)?.clone();
        group_tail(TailPlace::List, title, rows, currency, group_buy)
    };
    let stack_sections: Vec<(DeckSection, Vec<RowInfo>)> = stack_groups
        .into_iter()
        .map(
            |StackGroup {
                 key,
                 title,
                 pips,
                 rows,
             }| {
                let cards = rows.iter().map(|row| row.card.clone()).collect();
                (
                    DeckSection {
                        key,
                        title,
                        pips,
                        cards,
                    },
                    rows,
                )
            },
        )
        .collect();
    let edit_actions = DeckEditActions {
        change_quantity: Callback::new(move |(card_id, delta, _record): (Uuid, i32, bool)| {
            driver.dispatch(Action::ChangeQuantity { card_id, delta });
        }),
        move_to_board: Callback::new(move |(card_id, board, _record): (Uuid, Board, bool)| {
            driver.dispatch(Action::MoveTo { card_id, board });
        }),
        toggle_mvp: Callback::new(move |(card_id, _starred): (Uuid, bool)| {
            driver.dispatch(Action::ToggleMvp {
                card_id,
                now: chrono::Utc::now(),
            });
        }),
    };
    let stack_host = DeckRowHost {
        on_image: Some(Callback::new(move |shown: (ScryfallData, usize)| {
            let mut preview_card = preview_card;
            preview_card.set(Some(shown));
        })),
        on_printing: Some(Callback::new(move |(card, slot): (Card, Option<Slot>)| {
            driver.dispatch(Action::OpenPrintings {
                card: Box::new(card),
                slot,
            });
        })),
        on_view_printing: None,
        describe_tag: Some(describe_tag),
        on_examples: Some(open_examples),
    };

    let open_printings = move |card: Card, slot: Option<Slot>| {
        driver.dispatch(Action::OpenPrintings {
            card: Box::new(card),
            slot,
        });
    };
    let toggle_section = move |key: &str| {
        let key = key.to_owned();
        move |()| driver.dispatch(Action::ToggleSection(key.clone()))
    };

    rsx! {
            div {
                class: "screen",
                // Any tap while the stacks show can reveal a card or close one.
                "data-settle": stacks_on.then_some(""),
                // A tap off every card closes the reveal, of a stack card or a featured card; a
                // card's own tap stops at the card.
                onclick: move |_| stack_details.dismiss(),
                ScreenHeader { title: deck_cards::TITLE, hint: deck_cards_hint_open, back: move |()| driver.dispatch(Action::Back) }

                div { class: "screen-content",

                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    // Deck identity: name + format/power/tag chips in one
                    // wrapping row, echoing the zite share page (same stat-chip
                    // accents).
                    if !deck_name.is_empty() {
                        div { class: "deck-cards-header row-enter",
                            span { class: "deck-cards-header-name", "{deck_name}" }
                            if let Some(fmt) = deck_format {
                                span { class: "stat-chip stat-chip-p3", "{fmt.display_name()}" }
                            }
                            if let Some(pl) = deck_power {
                                span { class: "stat-chip stat-chip-p5", "{pl.display_name()}" }
                            }
                            for tag in deck_tags {
                                span { key: "{tag}", class: "stat-chip stat-chip-muted", "{deck::deck_tag_label(&tag)}" }
                            }
                            for tag in deck_other_tags {
                                span { key: "{tag}", class: "stat-chip stat-chip-p5", "{tag.display_name()}" }
                            }
                        }
                    } else if !loaded {
                        // Identity ghost in the header's own spot until the
                        // profile lands.
                        div { class: "deck-cards-header",
                            div { class: "skeleton-bar skeleton-deck-list-title" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                        }
                    }

                    // No ghost for the featured strip: it holds anywhere from
                    // zero to six cards, so any fixed count is wrong most of
                    // the time and resolving to fewer reads as cards vanishing.
                    // The cards deal themselves in from above instead, which
                    // already carries the arrival. The rest of the screen keeps
                    // its skeleton.
                    FeaturedCards {
                        cards: featured_cards,
                        loaded,
                        // The first tap opens the card's details, as a click does on the web:
                        // nothing covers a featured card, so there is nothing to float clear
                        // first. A card no listed row has opens its image.
                        on_tap: move |card: Card| {
                            let id = card.scryfall_data.id;
                            match featured_open(id, &listed_ids.peek()) {
                                FeaturedOpen::Details => stack_details.feature(id),
                                FeaturedOpen::Image => {
                                    let mut preview = preview_card;
                                    preview.set(Some((card.scryfall_data, 0)));
                                }
                            }
                        },
                    }

                    // The controls ghost with the list until the deck lands: nothing
                    // here is usable before then.
                    if loaded {
                        // Quick-add: type a name, add straight to the mainboard.
                        QuickAdd {
                            query: quick_query,
                            results: quick_results,
                            oninput: move |query: String| driver.dispatch(Action::Type(query)),
                            onclear: move |()| driver.dispatch(Action::ClearQuickAdd),
                            onadd: move |card: Card| driver.dispatch(Action::AddCard(Box::new(card))),
                        }

                        // List or Stacks
                        CardsViewSwitch {
                            view,
                            on_change: move |choice: CardsView| cards_view_store::infallible_save(choice),
                        }

                        // Group-by row
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_cards::GROUP_BY}" }
                            for option in GroupByOption::all() {
                                Chip {
                                    key: "{option}",
                                    selected: group_by == option,
                                    onclick: move |_| driver.dispatch(Action::PickGroupBy(option)),
                                    "{option}"
                                }
                            }
                        }

                        // Board filter row (multi-select, at least one must be on)
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_cards::BOARDS}" }
                            Chip {
                                selected: show_deck,
                                onclick: move |_| driver.dispatch(Action::ToggleBoard(Board::Deck)),
                                "{deck_cards::board_label(Board::Deck)}"
                            }
                            Chip {
                                selected: show_maybe,
                                disabled: !maybe_enabled,
                                onclick: move |_| driver.dispatch(Action::ToggleBoard(Board::Maybeboard)),
                                "{deck_cards::board_label(Board::Maybeboard)}"
                            }
                            Chip {
                                selected: show_side,
                                disabled: !side_enabled,
                                onclick: move |_| driver.dispatch(Action::ToggleBoard(Board::Sideboard)),
                                "{deck_cards::board_label(Board::Sideboard)}"
                            }
                        }

                        // Show toggles row
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_cards::SHOW}" }
                            Chip {
                                selected: show_tokens,
                                onclick: move |_| driver.dispatch(Action::ToggleTokens),
                                "{deck_cards::TOKENS}"
                            }
                            if offers_command_zone {
                                Chip {
                                    selected: show_command_zone,
                                    onclick: move |_| driver.dispatch(Action::ToggleCommandZone),
                                    "{deck_cards::COMMAND_ZONE}"
                                }
                            }
                            Chip {
                                selected: show_row_art(),
                                disabled: stacks_on,
                                onclick: move |_| show_row_art.set(!show_row_art()),
                                "{deck_cards::ART}"
                            }
                            Button {
                                variant: ButtonVariant::Action,
                                class: "btn-icon deck-fold-all",
                                onclick: move |_| {
                                    // Folding every stack can move stacks between the columns,
                                    // and the settle skips this button, so it starts the watch.
                                    if stacks_on {
                                        components::settle::watch();
                                    }
                                    driver.dispatch(Action::SetSections {
                                        keys: section_keys.clone(),
                                        collapsed: !all_collapsed,
                                    });
                                },
                                icons::DockIcon { path: if all_collapsed { icons::UNFOLD } else { icons::FOLD } }

                                span { class: "sr-only", if all_collapsed { "Open every group" } else { "Shut every group" } }
                            }
                        }
                    } else {
                        QuickAddSkeleton { label: deck_cards::QUICK_ADD }
                        ChipRowGhost {
                            label: deck_cards::GROUP_BY,
                            chips: GroupByOption::all().into_iter().map(|words| ChipGhost::Text(words.to_string())).collect::<Vec<_>>(),
                        }
                        ChipRowGhost {
                            label: deck_cards::BOARDS,
                            chips: [Board::Deck, Board::Maybeboard, Board::Sideboard]
                                .map(|board| ChipGhost::Text(deck_cards::board_label(board).to_string()))
                                .to_vec(),
                        }
                        ChipRowGhost {
                            label: deck_cards::SHOW,
                            chips: [deck_cards::TOKENS, deck_cards::COMMAND_ZONE, deck_cards::ART]
                                .map(|words| ChipGhost::Text(words.to_string()))
                                .to_vec(),
                        }
                        // The cards' ghost follows the saved view.
                        if view() == CardsView::Stacks {
                            components::DeckStacksGhost { touch: true }
                        } else {
                            DeckCardListSkeleton {}
                        }
                    }

                    if stacks_on {
                        div { class: "app-deck-stacks", "data-settle-scope": "",
                            DeckStacks {
                                sections: stack_sections,
                                currency: price_budget_currency(),
                                collapsed: folds,
                                details: stack_details,
                                faces: stack_faces,
                                group_buy,
                                dealing: dealing(),
                                touch: true,
                                on_fold: move |key: String| driver.dispatch(Action::ToggleSection(key)),
                            }
                        }
                    } else {
                    // Token list
                    if show_tokens {
                        if let Some(tokens) = tokens {
                            if !tokens.is_empty() {
                                CardGroup {
                                    title: deck_cards::tokens_title(tokens.len()),
                                    expanded: tokens_expanded,
                                    ontoggle: toggle_section(deck_cards::TOKENS_KEY),
                                    trailing: tail(deck_cards::TOKENS_KEY),
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

                    // Maybeboard section
                    if show_maybe && !display.maybe.is_empty() {
                        CardGroup {
                            title: deck_cards::maybeboard_title(display.maybe.len()),
                            expanded: maybe_expanded,
                            ontoggle: toggle_section(deck_cards::MAYBEBOARD_KEY),
                            trailing: tail(deck_cards::MAYBEBOARD_KEY),
                            for entry in display.maybe.iter() {
                                {
                                    let card_id = entry.card.scryfall_data.id;
                                    let qty = *entry.deck_card.quantity;
                                    rsx! {
                                        CardRow {
                                            card: entry.card.clone(),
                                            qty,
                                            expanded_card,
                                            preview_card,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| driver.dispatch(Action::ChangeQuantity { card_id, delta }),
                                            on_move_to: move |board: Board| driver.dispatch(Action::MoveTo { card_id, board }),
                                            current_board: Some(Board::Maybeboard),
                                            on_printing: move |card: Card| open_printings(card, None),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Sideboard section
                    if show_side && !display.side.is_empty() {
                        CardGroup {
                            title: deck_cards::sideboard_title(display.side.len()),
                            expanded: side_expanded,
                            ontoggle: toggle_section(deck_cards::SIDEBOARD_KEY),
                            trailing: tail(deck_cards::SIDEBOARD_KEY),
                            for entry in display.side.iter() {
                                {
                                    let card_id = entry.card.scryfall_data.id;
                                    let qty = *entry.deck_card.quantity;
                                    rsx! {
                                        CardRow {
                                            card: entry.card.clone(),
                                            qty,
                                            expanded_card,
                                            preview_card,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| driver.dispatch(Action::ChangeQuantity { card_id, delta }),
                                            on_move_to: move |board: Board| driver.dispatch(Action::MoveTo { card_id, board }),
                                            current_board: Some(Board::Sideboard),
                                            on_printing: move |card: Card| open_printings(card, None),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if show_command_zone {
                        // Pinned commander/oathbreaker group (includes partner if present)
                        if displayed_commander.is_some() || has_partner {
                            CardGroup {
                                title: Slot::Commander.group_header(deck_format, has_partner),
                                expanded: commander_expanded,
                                ontoggle: toggle_section(Slot::Commander.group_key()),
                                trailing: tail(Slot::Commander.group_key()),
                                if let Some(cmd) = displayed_commander {
                                    CardRow {
                                        card: cmd,
                                        qty: 1,
                                        expanded_card,
                                        preview_card,
                                        preview_dismissing,
                                        on_printing: move |card: Card| open_printings(card, Some(Slot::Commander)),
                                    }
                                }
                                if let Some(partner) = displayed_partner {
                                    CardRow {
                                        card: partner,
                                        qty: 1,
                                        expanded_card,
                                        preview_card,
                                        preview_dismissing,
                                        on_printing: move |card: Card| open_printings(card, Some(Slot::Partner)),
                                    }
                                }
                            }
                        }

                        // Pinned background group
                        if let Some(bg) = displayed_background {
                            CardGroup {
                                title: Slot::Background.group_header(deck_format, has_partner),
                                expanded: background_expanded,
                                ontoggle: toggle_section(Slot::Background.group_key()),
                                trailing: tail(Slot::Background.group_key()),
                                CardRow {
                                    card: bg,
                                    qty: 1,
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_printing: move |card: Card| open_printings(card, Some(Slot::Background)),
                                }
                            }
                        }

                        // Pinned signature spell (Oathbreaker only)
                        if let Some(spell) = displayed_signature_spell {
                            CardGroup {
                                title: Slot::SignatureSpell.group_header(deck_format, has_partner),
                                expanded: spell_expanded,
                                ontoggle: toggle_section(Slot::SignatureSpell.group_key()),
                                trailing: tail(Slot::SignatureSpell.group_key()),
                                CardRow {
                                    card: spell,
                                    qty: 1,
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_printing: move |card: Card| open_printings(card, Some(Slot::SignatureSpell)),
                                }
                            }
                        }
                    }

                    // Card groups (active deck cards only)
                    if show_deck {
                        for (title, key, pips, group_expanded, rows) in groups {
                            CardGroup {
                                // Color groups show pips in place of a label, so the
                                // title is just the count; the key comes from the pips.
                                title,
                                pips,
                                expanded: group_expanded,
                                ontoggle: toggle_section(&key),
                                trailing: tail(&key),
                                for (card, row) in rows {
                                    {
                                        let card_id = card.scryfall_data.id;
                                        let qty = row.map_or(1, |r| r.quantity);
                                        let is_mvp = row.is_some_and(|r| r.mvp);
                                        rsx! {
                                            CardRow {
                                                card,
                                                qty,
                                                expanded_card,
                                                preview_card,
                                                preview_dismissing,
                                                on_qty_change: move |delta: i32| driver.dispatch(Action::ChangeQuantity { card_id, delta }),
                                                on_move_to: move |board: Board| driver.dispatch(Action::MoveTo { card_id, board }),
                                                current_board: row.map(|r| r.board),
                                                on_printing: move |card: Card| open_printings(card, None),
                                                mvp: Some(is_mvp),
                                                on_toggle_mvp: move |()| driver.dispatch(Action::ToggleMvp { card_id, now: chrono::Utc::now() }),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Lands section: pinned at the bottom of the mainboard, out of
                    // the group-by pipeline so it reads the same in every grouping
                    // mode. Hide lands via the card filter's Land type exclusion.
                    if show_deck && !lands.is_empty() {
                        CardGroup {
                            title: lands_title,
                            expanded: lands_expanded,
                            ontoggle: toggle_section(deck_cards::LANDS_KEY),
                            trailing: tail(deck_cards::LANDS_KEY),
                            for (card, row) in lands {
                                {
                                    let card_id = card.scryfall_data.id;
                                    let qty = row.map_or(1, |r| r.quantity);
                                    let is_mvp = row.is_some_and(|r| r.mvp);
                                    rsx! {
                                        CardRow {
                                            card,
                                            qty,
                                            expanded_card,
                                            preview_card,
                                            preview_dismissing,
                                            on_qty_change: move |delta: i32| driver.dispatch(Action::ChangeQuantity { card_id, delta }),
                                            on_move_to: move |board: Board| driver.dispatch(Action::MoveTo { card_id, board }),
                                            current_board: row.map(|r| r.board),
                                            on_printing: move |card: Card| open_printings(card, None),
                                            mvp: Some(is_mvp),
                                            on_toggle_mvp: move |()| driver.dispatch(Action::ToggleMvp { card_id, now: chrono::Utc::now() }),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    }

                    if no_cards {
                        p { class: "deck-empty",
                            span { class: "chip-note", "{deck_cards::NO_CARDS}" }
                        }
                    }

                }
            }

            // The screen's actions as one floating icon bar at the bottom right:
            // add and remove, then the filter (with its reset while a filter is
            // on) and undo.
            div { class: "action-dock action-dock-right",
                DockAction {
                    label: deck_cards::ADD,
                    icon: icons::PLUS,
                    onclick: move |()| driver.dispatch(Action::OpenAdd),
                }
                DockAction {
                    label: deck_cards::REMOVE,
                    icon: icons::MINUS,
                    onclick: move |()| driver.dispatch(Action::OpenRemove),
                }
                span { class: "action-dock-divider" }
                DockFilter {
                    label: deck_cards::FILTER,
                    dot: filter::shows_dot(&filter),
                    active: !filter.is_empty(),
                    onclick: move |()| filters_overlay_open.set(true),
                    on_reset: move |()| driver.dispatch(Action::ResetFilter),
                    reset_label: deck_cards::RESET_FILTER,
                }
                // Walks the undo log newest-first; disabled while there is
                // nothing to undo, so the pill keeps its width.
                DockAction {
                    label: deck_cards::UNDO,
                    icon: icons::UNDO,
                    disabled: undo_log.0.read().is_empty(),
                    onclick: move |()| {
                        let mut log = undo_log.0;
                        let popped = log.write().pop();
                        if let Some(action) = popped {
                            driver.dispatch(Action::Undo(Box::new(to_shared_undo(action))));
                        }
                    },
                }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
            }

            GroupBuyDialog { group_buy }
            // The sheet a stack card opens, and in either view a featured card.
            DeckStackDrawer {
                rows: stack_rows,
                boards: stack_boards,
                details: stack_details,
                faces: stack_faces,
                actions: Some(edit_actions),
                host: stack_host,
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }

            OneTimeHintDialog { open: deck_cards_hint_open, hint: OneTimeHint::DeckCards }

            if let Some(card) = printing_card {
                PrintingSheet {
                    host_screen: screen::DECK_CARD_VIEW,
                    card,
                    open: printing_sheet_open,
                    on_save: move |new_card: Card| driver.dispatch(Action::PickPrinting(Box::new(new_card))),
                }
            }

            if otag_examples_open() {
                OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
            }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures;

    #[test]
    fn undo_entries_survive_the_round_trip_to_the_model() {
        let entry = test_fixtures::deck_entry("Sol Ring", 2);
        let card_id = entry.card.scryfall_data.id;
        let removed = to_app_undo(to_shared_undo(UndoAction::Removed {
            entry: entry.clone(),
            baseline: 3,
        }));
        assert!(
            matches!(&removed, UndoAction::Removed { entry: e, baseline: 3 } if *e == entry),
            "{card_id}"
        );
        let moved = to_app_undo(to_shared_undo(UndoAction::MovedBoard {
            card_id,
            card_name: "Sol Ring".to_owned(),
            from: Board::Sideboard,
        }));
        assert!(matches!(
            moved,
            UndoAction::MovedBoard { card_id: c, from: Board::Sideboard, .. } if c == card_id
        ));
    }
}
