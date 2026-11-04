//! Remove cards: swipe through the deck's cards to trim it.
//!
//! Right removes every copy, left keeps the card, up moves it between the
//! main deck and the maybeboard, down undoes, by drag, the arrow keys or the
//! four buttons under the card. The `Boards` row picks which board is
//! trimmed. The columns and the docked filter follow Add; Ctrl+Z also
//! undoes, F flips, P opens the printings to look at and R is `Refresh`.

use components::icons::REFRESH;
use components::{Chip, DockFilter, DockIcon, EyeIcon, TOAST_QUICK};
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::deck_metrics::deck_price;
use zwipe_core::domain::deck::{Board, DeckEntry, DeckProfile};
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_ui::deck_signals::{Crossing, budget_crossing, land_count, land_crossing};
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::screens::remove_cards;
use zwipe_ui::stack::{CardStack, RemoveAction, StackAction};
use zwipe_ui::stores::filters::FilterScope;
use zwipe_ui::stores::undo::{UndoAction, create_board_field};

use super::{SwipeDetails, flip_top_card, signal_toast};
use crate::inbound::card_filter::{CardFilterPanel, use_filter_docked};
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_info::{CardDetailsSheet, CardInfoDisplay, CardSkeleton};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::swipe_stack::{
    Swipe, SwipeButtons, SwipeStack, use_swipe_control, use_swipe_keys_or_undo,
};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::chrome::page_keys::{use_page_keys, use_undo_key};
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::inbound::shell::ScreenFrame;

/// Which board Remove trims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardFilter {
    /// The main deck.
    Deck,
    /// The maybeboard.
    Maybeboard,
    /// The sideboard.
    Sideboard,
    /// Every board.
    All,
}

impl BoardFilter {
    /// Whether an entry on `board` is in this view.
    fn holds(self, board: Board) -> bool {
        match self {
            Self::Deck => board.is_active(),
            Self::Maybeboard => board.is_maybeboard(),
            Self::Sideboard => board.is_sideboard(),
            Self::All => true,
        }
    }
}

/// The swipe directions Remove takes.
const ALL_SWIPES: [Swipe; 4] = [Swipe::Left, Swipe::Right, Swipe::Up, Swipe::Down];

/// The Remove cards screen for `deck_id`.
#[component]
pub fn RemoveDeckCard(deck_id: Uuid) -> Element {
    let toast = use_toast();
    let authed = use_authed();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let CardStores {
        mut filters,
        mut undo,
        ..
    } = use_context();

    let mut filter = use_signal(|| {
        filters
            .peek()
            .restore(FilterScope::Remove, deck_id)
            .unwrap_or_default()
    });
    let mut entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);
    let mut zone_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut profile: Signal<Option<DeckProfile>> = use_signal(|| None);
    let mut loaded = use_signal(|| false);
    let mut stack: Signal<CardStack<RemoveAction>> = use_signal(CardStack::default);
    let mut board_filter = use_signal(|| BoardFilter::Deck);
    let mut counter = use_signal(|| 0u32);
    let mut show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);
    let mut filter_open = use_signal(|| false);
    let mut hint_open = use_one_time_hint(OneTimeHint::RemoveDeckCards);
    let docked = use_filter_docked();
    let control = use_swipe_control();

    let mut bump = move || {
        let next = counter.peek().wrapping_add(1);
        counter.set(next);
    };
    let currency = move || -> PriceCurrency {
        profile
            .peek()
            .as_ref()
            .and_then(|p| p.price_target_currency)
            .unwrap_or_default()
    };

    use_drop(move || {
        if let Ok(applied) = filter.try_peek() {
            filters
                .write()
                .park(FilterScope::Remove, deck_id, applied.clone());
        }
    });

    use_hook(move || {
        spawn(async move {
            let Some(deck) = authed
                .run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
            else {
                return;
            };
            zone_cards.set(deck.command_zone_cards);
            profile.set(Some(deck.deck_profile));
            entries.set(deck.entries);
            loaded.set(true);
            let next = counter.peek().wrapping_add(1);
            counter.set(next);
        });
    });

    // The pass: the picked board, filtered and sorted in memory.
    use_effect(move || {
        let _bumped = counter();
        let view = board_filter();
        if !*loaded.peek() {
            return;
        }
        let cards: Vec<Card> = entries
            .peek()
            .iter()
            .filter(|e| view.holds(e.deck_card.board))
            .map(|e| e.card.clone())
            .collect();
        let applied = filter.peek().clone();
        let mut shown = Cards::from(cards);
        if !applied.is_empty()
            && let Ok(criteria) = applied.build_criteria()
        {
            shown = shown.matching(&criteria);
        }
        stack
            .write()
            .replace(shown.sorted(applied.sort(), applied.direction()).into());
    });

    // The land toast on a downward crossing and the budget toast, from the
    // deck as it stood `before`.
    let crossings = move |before: &[DeckEntry]| {
        let after = entries.peek().clone();
        let cur = currency();
        let target = profile.peek().as_ref().and_then(|p| p.land_target);
        let budget = profile.peek().as_ref().and_then(|p| p.price_target);
        let zone_now = zone_cards.peek().clone();
        signal_toast(
            toast,
            land_crossing(
                target,
                land_count(before),
                land_count(&after),
                Crossing::Down,
            ),
        );
        signal_toast(
            toast,
            budget_crossing(
                budget,
                cur,
                deck_price(before, &zone_now, cur),
                deck_price(&after, &zone_now, cur),
            ),
        );
    };

    let undo_last = use_callback(move |()| {
        let popped = stack.write().pop_action();
        let Some(action) = popped else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };
        let quick = ToastOptions::default().duration(TOAST_QUICK);
        stack.write().prime_entering(action.exited());
        match action {
            RemoveAction::Keep => {
                if !stack.write().retreat_wrapping() {
                    stack.write().cancel_entering();
                }
                toast.info("Undid keep".to_string(), quick);
            }
            RemoveAction::Remove {
                card,
                board,
                quantity,
            } => {
                let card = *card;
                let card_id = card.scryfall_data.id;
                let taken = undo.write().take_newest(deck_id, |a| {
                    matches!(a, UndoAction::Removed { entry, .. } if entry.deck_card.scryfall_data_id == card_id)
                });
                let back = entries
                    .peek()
                    .iter()
                    .any(|e| e.deck_card.scryfall_data_id == card_id);
                if taken.is_none() && back {
                    stack.write().cancel_entering();
                    toast.info("Already changed".to_string(), quick);
                    return;
                }
                stack.write().insert_current(card.clone());
                spawn(async move {
                    let fresh = match ensure_fresh(session, client).await {
                        Ok(fresh) => fresh,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            stack.write().cancel_entering();
                            if let Some(entry) = taken {
                                undo.write().push(deck_id, entry);
                            }
                            return;
                        }
                    };
                    let request = HttpCreateDeckCard::new(
                        &card.scryfall_data,
                        quantity,
                        create_board_field(board),
                    );
                    let http = client.peek().clone();
                    match http.create_deck_card(deck_id, &request, &fresh).await {
                        Ok(deck_card) => {
                            entries.write().push(DeckEntry { card, deck_card });
                            toast.success("Undid remove".to_string(), quick);
                        }
                        Err(e) => {
                            toast.error(format!("Failed to undo: {e}"), ToastOptions::default());
                            if let Some(entry) = taken {
                                undo.write().push(deck_id, entry);
                            }
                        }
                    }
                });
            }
            RemoveAction::MoveBoard { card, from, .. } => {
                let card = *card;
                let card_id = card.scryfall_data.id;
                let taken = undo.write().take_newest(deck_id, |a| {
                    matches!(a, UndoAction::MovedBoard { card_id: id, from: f, .. } if *id == card_id && *f == from)
                });
                let back = entries
                    .peek()
                    .iter()
                    .any(|e| e.deck_card.scryfall_data_id == card_id && e.deck_card.board == from);
                if taken.is_none() && back {
                    stack.write().cancel_entering();
                    toast.info("Already changed".to_string(), quick);
                    return;
                }
                stack.write().insert_current(card);
                spawn(async move {
                    let fresh = match ensure_fresh(session, client).await {
                        Ok(fresh) => fresh,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            stack.write().cancel_entering();
                            if let Some(entry) = taken {
                                undo.write().push(deck_id, entry);
                            }
                            return;
                        }
                    };
                    let request =
                        HttpPatchDeckCard::new(None, Some(from.display_name().to_owned()));
                    let http = client.peek().clone();
                    match http
                        .update_deck_card(deck_id, card_id, &request, &fresh)
                        .await
                    {
                        Ok(_) => {
                            if let Some(e) = entries
                                .write()
                                .iter_mut()
                                .find(|e| e.deck_card.scryfall_data_id == card_id)
                            {
                                e.deck_card.board = from;
                            }
                            toast.success("Undid move".to_string(), quick);
                        }
                        Err(e) => {
                            toast.error(format!("Failed to undo: {e}"), ToastOptions::default());
                            if let Some(entry) = taken {
                                undo.write().push(deck_id, entry);
                            }
                        }
                    }
                });
            }
        }
    });

    let mut on_swipe = move |(swipe, card): (Swipe, Card)| {
        let quick = ToastOptions::default().duration(TOAST_QUICK);
        let card_id = card.scryfall_data.id;
        let entry = entries
            .peek()
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == card_id)
            .cloned();
        match swipe {
            Swipe::Left => {
                stack.write().record(RemoveAction::Keep);
                toast.info("Kept".to_string(), quick);
                stack.write().advance_wrapping();
            }
            Swipe::Right => {
                let (board, quantity) = entry.as_ref().map_or((Board::Deck, 1), |e| {
                    (e.deck_card.board, *e.deck_card.quantity)
                });
                stack.write().record(RemoveAction::Remove {
                    card: Box::new(card),
                    board,
                    quantity,
                });
                if let Some(entry) = entry {
                    spawn_forever(async move {
                        if authed
                            .run("remove_card", |c, s| async move {
                                c.delete_deck_card(deck_id, card_id, &s).await
                            })
                            .await
                            .is_some()
                        {
                            let baseline = *entry.deck_card.quantity;
                            undo.write().push(
                                deck_id,
                                UndoAction::Removed {
                                    entry: Box::new(entry),
                                    baseline,
                                },
                            );
                        }
                    });
                }
                toast.success("Removed from deck".to_string(), quick);
                let before = entries.peek().clone();
                entries
                    .write()
                    .retain(|e| e.deck_card.scryfall_data_id != card_id);
                stack.write().remove_current();
                crossings(&before);
            }
            Swipe::Up => {
                let from = entry.as_ref().map_or(Board::Deck, |e| e.deck_card.board);
                let to = if from.is_maybeboard() {
                    Board::Deck
                } else {
                    Board::Maybeboard
                };
                stack.write().record(RemoveAction::MoveBoard {
                    card: Box::new(card.clone()),
                    from,
                    to,
                });
                let card_name = card.scryfall_data.name;
                spawn_forever(async move {
                    let request = HttpPatchDeckCard::new(None, Some(to.display_name().to_owned()));
                    if authed
                        .run("move_card", |c, s| async move {
                            c.update_deck_card(deck_id, card_id, &request, &s).await
                        })
                        .await
                        .is_some()
                    {
                        undo.write().push(
                            deck_id,
                            UndoAction::MovedBoard {
                                card_id,
                                card_name,
                                from,
                            },
                        );
                    }
                });
                let message = if to == Board::Deck {
                    "Moved to main"
                } else {
                    "Moved to maybeboard"
                };
                toast.info(message.to_string(), quick);
                let before = entries.peek().clone();
                if let Some(e) = entries
                    .write()
                    .iter_mut()
                    .find(|e| e.deck_card.scryfall_data_id == card_id)
                {
                    e.deck_card.board = to;
                }
                stack.write().remove_current();
                crossings(&before);
            }
            Swipe::Down => undo_last.call(()),
        }
    };

    let refresh = use_callback(move |()| {
        stack.write().rewind();
        if filter.peek().sort() == Some(CardSortKey::Random) {
            bump();
        }
        toast.info(
            "Stack refreshed".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    });

    use_swipe_keys_or_undo(
        control,
        ALL_SWIPES.to_vec(),
        Callback::new(move |()| stack.peek().current_wrapping().is_some()),
        undo_last,
    );
    use_undo_key(Callback::new(move |()| undo_last.call(())));
    use_page_keys(Callback::new(move |key: String| match key.as_str() {
        "f" | "F" => flip_top_card(),
        "p" | "P" => {
            if stack.peek().current_wrapping().is_some() {
                printing_open.set(true);
            }
        }
        "r" | "R" => refresh.call(()),
        _ => {}
    }));

    let current = stack.read().current_wrapping().cloned();
    let (window, entering) = {
        let s = stack.read();
        (s.window().to_vec(), s.entering())
    };
    let view = board_filter();
    let dot = filter::shows_dot(&filter.read());
    let sources = Sources::deck(
        &entries
            .read()
            .iter()
            .map(|e| e.card.clone())
            .collect::<Vec<_>>(),
    );
    let panel_config = SheetConfig {
        show_format: true,
        sources,
        ..SheetConfig::default()
    };
    let on_apply = move |applied: CardQueryBuilder| {
        filter.set(applied);
        bump();
    };
    let buttons = vec![
        (Swipe::Left, "Keep"),
        (Swipe::Up, "Move"),
        (Swipe::Down, "Undo"),
        (Swipe::Right, "Remove"),
    ];

    rsx! {
        ScreenFrame {
            title: remove_cards::TITLE,
            description: Some("Swipe through the deck's cards to trim it".to_string()),
            on_hint: move |()| hint_open.set(true),
            div { class: "screen-content card-swipe content-enter web-cards-swipe",
                div { class: "web-cards-swipe-main",
                    div { class: "form-container",
                        if let Some(card) = current.clone() {
                            SwipeStack {
                                cards: window,
                                allowed: ALL_SWIPES.to_vec(),
                                control,
                                entering,
                                on_entered: move |()| stack.write().cancel_entering(),
                                on_swipe: move |pair: (Swipe, Card)| on_swipe(pair),
                            }
                            SwipeButtons { control, buttons }
                            CardInfoDisplay { card }
                        } else {
                            CardSkeleton {}
                        }
                    }
                }
                SwipeDetails { card: current.clone() }
                if docked() {
                    CardFilterPanel {
                        open: filter_open,
                        filter,
                        config: panel_config.clone(),
                        on_apply,
                        docked: Some(docked),
                    }
                }
            }
            // The bottom row: the board chips on the left, the action dock
            // (filter, refresh, card details) on the right. Back is the
            // header's arrow.
            div { class: "swipe-bottom-row",
                div { class: "swipe-bottom-chips",
                    for (label, variant) in [
                        ("Main", BoardFilter::Deck),
                        ("Maybe", BoardFilter::Maybeboard),
                        ("Side", BoardFilter::Sideboard),
                        ("All", BoardFilter::All),
                    ] {
                        Chip {
                            key: "{label}",
                            selected: view == variant,
                            onclick: move |_| {
                                board_filter.set(variant);
                                bump();
                            },
                            "{label}"
                        }
                    }
                }
                div { class: "action-dock",
                    if !docked() {
                        DockFilter {
                            label: "Filter",
                            dot,
                            onclick: move |()| filter_open.set(true),
                        }
                    }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Refresh",
                        onclick: move |_| refresh.call(()),
                        DockIcon { path: REFRESH }
                    }
                    span { class: "action-dock-divider" }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Card details",
                        disabled: current.is_none(),
                        onclick: move |_| show_details.toggle(),
                        EyeIcon {}
                    }
                }
            }
        }
        if let Some(card) = current {
            SheetFrame { form: SheetForm::Modal,
                CardDetailsSheet {
                    open: show_details,
                    card: card.clone(),
                    on_printings: move |()| printing_open.set(true),
                }
            }
            PrintingSheet {
                card,
                open: printing_open,
                read_only: true,
                on_save: move |_: Card| {},
            }
        }
        if !docked() {
            CardFilterPanel {
                open: filter_open,
                filter,
                config: panel_config,
                on_apply,
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::RemoveDeckCards, open: hint_open }
    }
}
