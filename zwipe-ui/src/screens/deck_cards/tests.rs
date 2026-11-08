use chrono::Duration as ChronoDuration;
use zwipe_core::domain::deck::Format;
use zwipe_core::http::helpers::Patch;

use super::*;
use crate::test_support::{card, deck, deck_profile};
use crate::toast::{NORMAL, QUICK};

fn typed(name: &str, type_line: &str) -> Card {
    let mut c = card(name);
    c.scryfall_data.type_line = Some(type_line.to_owned());
    c
}

fn id(card: &Card) -> Uuid {
    card.scryfall_data.id
}

fn blank() -> CardQueryBuilder {
    CardQueryBuilder::default()
}

fn toasts(effects: &[Effect]) -> Vec<(ToastKind, String)> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::Toast(t) => Some((t.kind, t.text.clone())),
            _ => None,
        })
        .collect()
}

fn undo_records(effects: &[Effect]) -> Vec<UndoAction> {
    effects
        .iter()
        .filter_map(|e| match e {
            Effect::RecordUndo(a) => Some((**a).clone()),
            _ => None,
        })
        .collect()
}

fn debounce(effects: &[Effect]) -> Ticket {
    effects
        .iter()
        .find_map(|e| match e {
            Effect::Debounce { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .expect("a debounce")
}

/// A state whose load is out, with the deck and tokens tickets.
fn loading(filter: CardQueryBuilder) -> (State, Uuid, Ticket, Ticket) {
    let deck_id = Uuid::new_v4();
    let mut state = State::new();
    let effects = state.update(Action::Load {
        deck_id,
        filter: Box::new(filter),
    });
    let [
        Effect::FetchDeck {
            ticket: deck_ticket,
            deck_id: d1,
        },
        Effect::FetchTokens {
            ticket: tokens_ticket,
            deck_id: d2,
        },
    ] = effects.as_slice()
    else {
        panic!("Load fetches the deck and tokens: {effects:?}");
    };
    assert_eq!((*d1, *d2), (deck_id, deck_id));
    (state, deck_id, *deck_ticket, *tokens_ticket)
}

/// Sol Ring and Forest on the main board, Maybe on the maybeboard, Side on the sideboard.
fn library() -> Vec<Card> {
    vec![
        typed("Sol Ring", "Artifact"),
        typed("Forest", "Basic Land — Forest"),
        typed("Maybe", "Instant"),
        typed("Side", "Sorcery"),
    ]
}

/// A loaded state holding `cards` (the third on the maybeboard and the fourth on the sideboard, when there), with its profile.
fn loaded_with(cards: Vec<Card>, profile: DeckProfile) -> (State, Vec<Effect>) {
    let (mut state, _, ticket, _) = loading(blank());
    let mut d = deck(profile.clone(), cards);
    if let Some(e) = d.entries.get_mut(2) {
        e.deck_card.board = Board::Maybeboard;
    }
    if let Some(e) = d.entries.get_mut(3) {
        e.deck_card.board = Board::Sideboard;
    }
    assert!(
        state
            .update(Action::ProfileLoaded {
                ticket,
                profile: Box::new(profile),
            })
            .is_empty()
    );
    let effects = state.update(Action::Loaded {
        ticket,
        deck: Box::new(d),
        zone: Box::default(),
    });
    (state, effects)
}

fn loaded() -> (State, Vec<Card>) {
    let cards = library();
    let (state, _) = loaded_with(cards.clone(), deck_profile("Deck"));
    (state, cards)
}

#[test]
fn the_screen_starts_loading_with_main_and_the_command_zone_on() {
    let state = State::new();
    assert!(!state.is_loaded());
    assert_eq!(state.group_by(), GroupByOption::CardType);
    assert_eq!(state.boards(), Boards::default());
    assert!(state.show_command_zone());
    assert!(!state.show_tokens());
    assert!(state.profile().is_none());
    assert!(state.quick_add().is_none());
    assert!(state.tokens(&blank()).is_none());
}

#[test]
fn a_load_lands_the_deck_offers_the_hint_and_refilters() {
    let (state, effects) = loaded_with(library(), deck_profile("Deck"));
    assert!(state.is_loaded());
    assert_eq!(
        state.profile().map(|p| p.name.to_string()),
        Some("Deck".to_owned())
    );
    assert!(matches!(
        effects.as_slice(),
        [Effect::Refilter, Effect::OfferHint(OneTimeHint::DeckCards)]
    ));
    assert_eq!(state.filter_cards().len(), 4);
    let display = state.display(&blank());
    assert_eq!(display.lands.len(), 1);
    assert_eq!(display.maybe.len(), 1);
    assert!(!state.no_cards(&display));
}

#[test]
fn an_empty_deck_says_no_cards_and_saves_the_hint() {
    let (state, effects) = loaded_with(Vec::new(), deck_profile("Empty"));
    assert!(matches!(effects.as_slice(), [Effect::Refilter]));
    assert!(state.no_cards(&state.display(&blank())));
}

#[test]
fn the_hint_waits_for_the_first_card() {
    let (mut state, _) = loaded_with(Vec::new(), deck_profile("Empty"));
    let result = card("Sol Ring");
    let effects = state.update(Action::AddCard(Box::new(result.clone())));
    let [Effect::CreateCard { ticket, .. }] = effects.as_slice() else {
        panic!("an add creates the card");
    };
    let ticket = *ticket;
    let created = deck(deck_profile("Empty"), vec![result]).entries.remove(0);
    let effects = state.update(Action::Created {
        ticket,
        deck_card: Box::new(created.deck_card),
    });
    assert!(matches!(
        effects.last(),
        Some(Effect::OfferHint(OneTimeHint::DeckCards))
    ));
}

#[test]
fn a_gone_deck_is_replaced_by_the_deck_list() {
    let (mut state, _, ticket, _) = loading(blank());
    assert!(matches!(
        state.update(Action::Gone { ticket }).as_slice(),
        [Effect::Replace(Destination::DeckList)]
    ));
    assert!(!state.is_loaded());
}

#[test]
fn a_failed_load_keeps_the_skeletons() {
    let (mut state, _, ticket, _) = loading(blank());
    assert!(state.update(Action::Failed { ticket }).is_empty());
    assert!(!state.is_loaded());
    assert!(!state.no_cards(&state.display(&blank())));
}

#[test]
fn a_remembered_filter_warns_once_the_deck_lands() {
    let (mut state, _, ticket, _) = loading(CardQueryBuilder::with_name_contains("ring"));
    let effects = state.update(Action::Loaded {
        ticket,
        deck: Box::new(deck(deck_profile("D"), library())),
        zone: Box::default(),
    });
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Warning, FILTER_ACTIVE.to_owned())]
    );
    let Some(Effect::Toast(toast)) = effects.first() else {
        panic!("the toast leads");
    };
    assert_eq!(toast.duration, QUICK);
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, _, old, old_tokens) = loading(blank());
    let _ = state.update(Action::Load {
        deck_id: Uuid::new_v4(),
        filter: Box::new(blank()),
    });
    let _ = state.update(Action::ProfileLoaded {
        ticket: old,
        profile: Box::new(deck_profile("Old")),
    });
    let effects = state.update(Action::Loaded {
        ticket: old,
        deck: Box::new(deck(deck_profile("Old"), library())),
        zone: Box::default(),
    });
    assert!(effects.is_empty());
    assert!(!state.is_loaded());
    assert!(state.profile().is_none());
    let _ = state.update(Action::TokensLoaded {
        ticket: old_tokens,
        tokens: vec![card("Treasure")],
    });
    assert!(state.tokens(&blank()).is_none());
}

#[test]
fn the_zone_leaves_the_lists_and_leads_the_featured_strip() {
    let cards = library();
    let mut profile = deck_profile("D");
    profile.format = Some(Format::Oathbreaker);
    let (mut state, _, ticket, _) = loading(blank());
    let _ = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile.clone()),
    });
    let mut d = deck(profile, cards.clone());
    d.entries[0].deck_card.mvp_at = Some(Utc::now());
    let zone = Zone {
        commander: Some(cards[1].clone()),
        ..Zone::default()
    };
    let _ = state.update(Action::Loaded {
        ticket,
        deck: Box::new(d),
        zone: Box::new(zone),
    });
    let display = state.display(&blank());
    assert!(display.lands.is_empty(), "the Forest sits in the zone");
    assert_eq!(display.zone.len(), 1);
    let featured: Vec<(String, &str)> = state
        .featured()
        .into_iter()
        .map(|(c, r)| (c.scryfall_data.name, r))
        .collect();
    assert_eq!(
        featured,
        [
            ("Forest".to_owned(), "Oathbreaker"),
            ("Sol Ring".to_owned(), MVP)
        ]
    );
}

#[test]
fn zone_requests_follow_the_fetch_order() {
    let mut profile = deck_profile("D");
    let (c, p, b, s) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    profile.commander_id = Some(c);
    profile.partner_commander_id = Some(p);
    profile.background_id = Some(b);
    profile.signature_spell_id = Some(s);
    assert_eq!(
        zone_requests(&profile),
        [
            (Slot::Commander, c),
            (Slot::SignatureSpell, s),
            (Slot::Partner, p),
            (Slot::Background, b)
        ]
    );
    assert_eq!(zone_requests(&deck_profile("none")), []);
}

#[test]
fn tokens_land_sorted_and_a_failure_reads_as_none() {
    let (mut state, _, _, tokens) = loading(blank());
    let _ = state.update(Action::TokensLoaded {
        ticket: tokens,
        tokens: vec![card("Treasure"), card("Clue")],
    });
    let names: Vec<String> = state
        .tokens(&blank())
        .unwrap()
        .into_iter()
        .map(|c| c.scryfall_data.name)
        .collect();
    assert_eq!(names, ["Clue", "Treasure"]);

    let (mut state, _, _, tokens) = loading(blank());
    let _ = state.update(Action::Failed { ticket: tokens });
    assert_eq!(state.tokens(&blank()), Some(Vec::new()));
}

#[test]
fn group_by_collapses_the_expanded_row() {
    let (mut state, _) = loaded();
    let effects = state.update(Action::PickGroupBy(GroupByOption::Cmc));
    assert!(matches!(effects.as_slice(), [Effect::Collapse]));
    assert_eq!(state.group_by(), GroupByOption::Cmc);
}

#[test]
fn boards_keep_one_on_and_gray_out_when_empty() {
    let (mut state, _) = loaded_with(vec![card("Main")], deck_profile("D"));
    assert!(!state.board_enabled(Board::Maybeboard));
    assert!(state.board_enabled(Board::Deck));
    let effects = state.update(Action::ToggleBoard(Board::Deck));
    assert!(matches!(
        effects.as_slice(),
        [Effect::Collapse, Effect::Refilter]
    ));
    assert!(state.board_on(Board::Deck), "the last board snaps back on");

    let (mut state, _) = loaded();
    assert!(state.board_enabled(Board::Maybeboard));
    let _ = state.update(Action::ToggleBoard(Board::Maybeboard));
    let _ = state.update(Action::ToggleBoard(Board::Deck));
    assert!(!state.board_on(Board::Deck));
    assert!(!state.no_cards(&state.display(&blank())), "Main is off");
}

#[test]
fn show_chips_and_sections_toggle() {
    let (mut state, _) = loaded();
    assert!(state.update(Action::ToggleTokens).is_empty());
    assert!(state.show_tokens());
    let _ = state.update(Action::ToggleCommandZone);
    assert!(!state.show_command_zone());
    let _ = state.update(Action::ToggleSection(LANDS_KEY.to_owned()));
    let _ = state.update(Action::ToggleSection(MAYBEBOARD_KEY.to_owned()));
    assert!(state.is_collapsed(LANDS_KEY) && state.is_collapsed(MAYBEBOARD_KEY));
    let _ = state.update(Action::ToggleSection(LANDS_KEY.to_owned()));
    assert!(!state.is_collapsed(LANDS_KEY));
}

#[test]
fn fold_all_collapses_every_section_and_opens_them_again() {
    let (mut state, _) = loaded();
    let keys = vec![LANDS_KEY.to_owned(), MAYBEBOARD_KEY.to_owned()];
    assert!(!state.all_collapsed(&keys));
    assert!(
        state
            .update(Action::SetSections {
                keys: keys.clone(),
                collapsed: true,
            })
            .is_empty()
    );
    assert!(state.all_collapsed(&keys));
    let _ = state.update(Action::ToggleSection(LANDS_KEY.to_owned()));
    assert!(!state.all_collapsed(&keys) && state.is_collapsed(MAYBEBOARD_KEY));
    let _ = state.update(Action::SetSections {
        keys,
        collapsed: false,
    });
    assert!(!state.is_collapsed(LANDS_KEY) && !state.is_collapsed(MAYBEBOARD_KEY));
    assert!(!state.all_collapsed(&[]));
}

#[test]
fn the_groups_follow_the_applied_filter_and_the_boards_the_live_one() {
    let (mut state, _) = loaded();
    let live = CardQueryBuilder::with_name_contains("zzz");
    let display = state.display(&live);
    assert_eq!(display.lands.len(), 1, "the groups wait for the apply");
    assert!(display.maybe.is_empty(), "the maybeboard follows the draft");
    assert!(
        state
            .update(Action::Filter(Box::new(live.clone())))
            .is_empty()
    );
    let display = state.display(&live);
    assert_eq!(display.lands, []);
    assert!(state.no_cards(&display));
}

#[test]
fn reset_filter_clears_collapses_and_toasts() {
    let (mut state, _) = loaded();
    let _ = state.update(Action::Filter(Box::new(
        CardQueryBuilder::with_name_contains("zzz"),
    )));
    let effects = state.update(Action::ResetFilter);
    assert!(matches!(
        effects.as_slice(),
        [
            Effect::ClearFilter,
            Effect::Collapse,
            Effect::Refilter,
            Effect::Toast(_)
        ]
    ));
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, FILTER_RESET.to_owned())]
    );
    assert_eq!(state.display(&blank()).lands.len(), 1);
}

#[test]
fn quick_add_debounces_searches_and_shows_results() {
    let (mut state, _) = loaded();
    assert!(state.update(Action::Type("s".to_owned())).is_empty());
    assert!(state.quick_add().is_none(), "one letter does not search");
    let effects = state.update(Action::Type("sol".to_owned()));
    let [Effect::Debounce { ticket, wait }] = effects.as_slice() else {
        panic!("a long enough query waits: {effects:?}");
    };
    assert_eq!(*wait, timing::QUICK_ADD_DEBOUNCE);
    assert_eq!(state.quick_add(), Some(QuickAddResults::Searching));
    let ticket = *ticket;
    let effects = state.update(Action::Debounced { ticket });
    let [
        Effect::SearchCards {
            ticket: searched, ..
        },
    ] = effects.as_slice()
    else {
        panic!("the debounce searches: {effects:?}");
    };
    assert_eq!(*searched, ticket);
    let _ = state.update(Action::Searched {
        ticket,
        cards: Vec::new(),
    });
    assert_eq!(state.quick_add(), Some(QuickAddResults::Empty));
}

#[test]
fn quick_add_query_turns_synergy_off_and_sorts_by_name() {
    let query = quick_add_query("  sol ");
    assert_eq!(query.name_contains(), Some("sol"));
    assert_eq!(query.sort(), Some(CardSortKey::Name));
}

#[test]
fn a_late_search_reply_is_ignored() {
    let (mut state, _) = loaded();
    let first = debounce(&state.update(Action::Type("sol".to_owned())));
    let _ = state.update(Action::Debounced { ticket: first });
    let second = debounce(&state.update(Action::Type("sol r".to_owned())));
    assert!(state.update(Action::Debounced { ticket: first }).is_empty());
    let _ = state.update(Action::Searched {
        ticket: first,
        cards: vec![card("Soldier")],
    });
    assert_eq!(state.quick_add(), Some(QuickAddResults::Searching));
    let _ = state.update(Action::Debounced { ticket: second });
    let _ = state.update(Action::Searched {
        ticket: second,
        cards: vec![card("Sol Ring")],
    });
    let Some(QuickAddResults::Found(found)) = state.quick_add() else {
        panic!("the newer reply shows");
    };
    assert_eq!(found[0].scryfall_data.name, "Sol Ring");
}

#[test]
fn a_failed_search_closes_and_clear_empties() {
    let (mut state, _) = loaded();
    let ticket = debounce(&state.update(Action::Type("sol".to_owned())));
    let _ = state.update(Action::Debounced { ticket });
    let _ = state.update(Action::Failed { ticket });
    assert!(state.quick_add().is_none());

    let ticket = debounce(&state.update(Action::Type("sol".to_owned())));
    let _ = state.update(Action::ClearQuickAdd);
    assert_eq!(state.quick_query(), "");
    assert!(state.quick_add().is_none());
    assert!(state.update(Action::Debounced { ticket }).is_empty());
}

#[test]
fn a_quick_add_joins_the_deck_records_undo_and_toasts() {
    let (mut state, _) = loaded();
    let ticket = debounce(&state.update(Action::Type("arc".to_owned())));
    let _ = state.update(Action::Debounced { ticket });
    let signet = card("Arcane Signet");
    let _ = state.update(Action::Searched {
        ticket,
        cards: vec![signet.clone()],
    });
    let effects = state.update(Action::AddCard(Box::new(signet.clone())));
    let [
        Effect::CreateCard {
            ticket,
            op: Op::QuickAddCard,
            request,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("an add creates the card: {effects:?}");
    };
    assert_eq!(request.quantity, 1);
    assert!(state.quick_add().is_none(), "the pick closes the results");
    assert_eq!(state.quick_query(), "", "and clears the search");
    let ticket = *ticket;
    let deck_card = deck(deck_profile("D"), vec![signet.clone()]).entries[0]
        .deck_card
        .clone();
    let effects = state.update(Action::Created {
        ticket,
        deck_card: Box::new(deck_card.clone()),
    });
    assert!(matches!(effects.first(), Some(Effect::Refilter)));
    assert_eq!(
        undo_records(&effects),
        [UndoAction::Added {
            card_id: id(&signet),
            card_name: "Arcane Signet".to_owned()
        }]
    );
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Added Arcane Signet".to_owned())]
    );
    assert!(state.row(id(&signet)).is_some());
    assert!(
        state
            .update(Action::Created {
                ticket,
                deck_card: Box::new(deck_card)
            })
            .is_empty(),
        "a reply lands once"
    );
}

#[test]
fn taps_fold_into_one_write_with_the_burst_net() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let first = debounce(&state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: 1,
    }));
    let effects = state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: 1,
    });
    assert!(matches!(effects.first(), Some(Effect::Refilter)));
    let second = debounce(&effects);
    assert_eq!(state.quantity(ring), 3, "the row changes at once");
    assert!(
        state.update(Action::Debounced { ticket: first }).is_empty(),
        "an older tap's debounce"
    );
    let effects = state.update(Action::Debounced { ticket: second });
    let [
        Effect::PatchCard {
            ticket,
            card_id,
            request,
            op: Op::ChangeQuantity,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("the burst writes: {effects:?}");
    };
    assert_eq!(*card_id, ring);
    assert!(matches!(request.quantity, Patch::Set(Some(3))));
    let ticket = *ticket;
    let deck_card = state.entries[0].deck_card.clone();
    let effects = state.update(Action::Patched {
        ticket,
        deck_card: Box::new(deck_card),
    });
    assert_eq!(
        undo_records(&effects),
        [UndoAction::QuantityChanged {
            card_id: ring,
            card_name: "Sol Ring".to_owned(),
            net: 2,
            baseline: 1
        }]
    );
}

#[test]
fn a_burst_that_nets_out_writes_nothing() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let _ = state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: 1,
    });
    let ticket = debounce(&state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: -1,
    }));
    assert!(state.update(Action::Debounced { ticket }).is_empty());
}

#[test]
fn a_tap_past_i32_is_dropped() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    assert!(
        state
            .update(Action::ChangeQuantity {
                card_id: ring,
                delta: i32::MAX,
            })
            .is_empty()
    );
    assert_eq!(state.quantity(ring), 1);
    assert!(state.update(Action::Leave).is_empty(), "no burst opened");
}

#[test]
fn a_failed_write_rolls_the_burst_back() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let ticket = debounce(&state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: 2,
    }));
    let effects = state.update(Action::Debounced { ticket });
    let [Effect::PatchCard { ticket, .. }] = effects.as_slice() else {
        panic!("the burst writes");
    };
    let effects = state.update(Action::Failed { ticket: *ticket });
    assert!(matches!(effects.as_slice(), [Effect::Refilter]));
    assert_eq!(state.quantity(ring), 1);
}

#[test]
fn a_tap_below_one_removes_the_row_and_records_it() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let effects = state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: -1,
    });
    assert!(matches!(
        effects.first(),
        Some(Effect::RecordRemoval { oracle_id, .. }) if *oracle_id == cards[0].scryfall_data.oracle_id
    ));
    let [UndoAction::Removed { baseline: 1, .. }] = undo_records(&effects).as_slice() else {
        panic!("the removal is undoable");
    };
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, CARD_REMOVED.to_owned())]
    );
    assert!(state.row(ring).is_none());
    let ticket = debounce(&effects);
    let effects = state.update(Action::Debounced { ticket });
    let [
        Effect::DeleteCard {
            ticket,
            op: Op::ChangeQuantity,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("the burst deletes: {effects:?}");
    };
    assert!(state.update(Action::Deleted { ticket: *ticket }).is_empty());
}

#[test]
fn a_failed_delete_puts_the_row_back_and_forgets_its_undo_entry() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let _ = state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: 2,
    });
    let ticket = debounce(&state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: -3,
    }));
    let effects = state.update(Action::Debounced { ticket });
    let [Effect::DeleteCard { ticket, .. }] = effects.as_slice() else {
        panic!("the burst deletes");
    };
    let effects = state.update(Action::Failed { ticket: *ticket });
    assert!(matches!(
        effects.as_slice(),
        [Effect::ForgetRemoval { card_id }, Effect::Refilter] if *card_id == ring
    ));
    assert_eq!(state.quantity(ring), 1, "back at its pre-burst quantity");
}

#[test]
fn land_and_budget_crossings_toast() {
    let cards = vec![typed("Forest", "Basic Land — Forest")];
    let mut profile = deck_profile("D");
    profile.land_target = Some(2);
    let (mut state, _) = loaded_with(cards.clone(), profile);
    let forest = id(&cards[0]);
    let effects = state.update(Action::ChangeQuantity {
        card_id: forest,
        delta: 1,
    });
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Land target reached (2)".to_owned())]
    );
    let Some(Effect::Toast(toast)) = effects.iter().find(|e| matches!(e, Effect::Toast(_))) else {
        panic!("a toast");
    };
    assert_eq!(toast.duration, NORMAL);
    let effects = state.update(Action::ChangeQuantity {
        card_id: forest,
        delta: -1,
    });
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Warning, "Below land target (2)".to_owned())]
    );

    let mut priced = card("Pricey");
    priced.scryfall_data.prices.usd = Some("30.00".to_owned());
    let mut profile = deck_profile("D");
    profile.price_target = Some(100.0);
    let (mut state, _) = loaded_with(vec![priced.clone()], profile);
    let effects = state.update(Action::ChangeQuantity {
        card_id: id(&priced),
        delta: 1,
    });
    assert_eq!(
        toasts(&effects),
        [(
            ToastKind::Info,
            "Deck at 60% of your $100.00 budget".to_owned()
        )]
    );
}

#[test]
fn leaving_flushes_every_waiting_burst() {
    let (mut state, cards) = loaded();
    let (ring, maybe) = (id(&cards[0]), id(&cards[2]));
    let _ = state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: 2,
    });
    let _ = state.update(Action::ChangeQuantity {
        card_id: maybe,
        delta: -1,
    });
    let effects = state.update(Action::Leave);
    let writes: Vec<(Uuid, Write)> = effects
        .iter()
        .filter_map(|e| match e {
            Effect::Flush { card_id, write, .. } => Some((*card_id, *write)),
            _ => None,
        })
        .collect();
    assert_eq!(writes, [(ring, Write::Set(3)), (maybe, Write::Delete)]);
    assert!(state.update(Action::Leave).is_empty());
}

#[test]
fn a_move_records_toasts_and_rolls_back_on_failure() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let effects = state.update(Action::MoveTo {
        card_id: ring,
        board: Board::Sideboard,
    });
    assert_eq!(
        undo_records(&effects),
        [UndoAction::MovedBoard {
            card_id: ring,
            card_name: "Sol Ring".to_owned(),
            from: Board::Deck
        }]
    );
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Moved to sideboard".to_owned())]
    );
    assert_eq!(state.row(ring).map(|r| r.board), Some(Board::Sideboard));
    let Some(Effect::PatchCard {
        ticket,
        request,
        op: Op::MoveBoard,
        ..
    }) = effects
        .iter()
        .find(|e| matches!(e, Effect::PatchCard { .. }))
    else {
        panic!("the move patches");
    };
    assert!(matches!(&request.board, Patch::Set(Some(b)) if b == "sideboard"));
    let _ = state.update(Action::Failed { ticket: *ticket });
    assert_eq!(state.row(ring).map(|r| r.board), Some(Board::Deck));
}

#[test]
fn a_star_adopts_the_servers_time_and_rolls_back_on_failure() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let now = Utc::now();
    let effects = state.update(Action::ToggleMvp { card_id: ring, now });
    let [
        Effect::Refilter,
        Effect::PatchCard {
            ticket, request, ..
        },
    ] = effects.as_slice()
    else {
        panic!("a star patches: {effects:?}");
    };
    assert!(matches!(request.mvp, Patch::Set(Some(true))));
    assert!(state.row(ring).unwrap().mvp);
    let server = now - ChronoDuration::seconds(5);
    let mut deck_card = state.entries[0].deck_card.clone();
    deck_card.mvp_at = Some(server);
    let _ = state.update(Action::Patched {
        ticket: *ticket,
        deck_card: Box::new(deck_card),
    });
    assert_eq!(state.entries[0].deck_card.mvp_at, Some(server));

    let effects = state.update(Action::ToggleMvp { card_id: ring, now });
    let Some(Effect::PatchCard { ticket, .. }) = effects.last() else {
        panic!("an unstar patches");
    };
    assert!(!state.row(ring).unwrap().mvp);
    let _ = state.update(Action::Failed { ticket: *ticket });
    assert_eq!(state.entries[0].deck_card.mvp_at, Some(server));
}

#[test]
fn a_printing_swaps_after_the_server_takes_it() {
    let (mut state, cards) = loaded();
    let old = cards[0].clone();
    let mut new = card("Sol Ring");
    new.scryfall_data.oracle_id = old.scryfall_data.oracle_id;
    let effects = state.update(Action::OpenPrintings {
        card: Box::new(old.clone()),
        slot: None,
    });
    assert!(matches!(effects.as_slice(), [Effect::OpenPrintings]));
    let effects = state.update(Action::PickPrinting(Box::new(new.clone())));
    let [
        Effect::PatchCard {
            ticket,
            card_id,
            op: Op::ChangePrinting,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("a printing patches the card: {effects:?}");
    };
    assert_eq!(*card_id, id(&old));
    assert!(
        state.row(id(&new)).is_none(),
        "nothing moves before the reply"
    );
    let ticket = *ticket;
    let effects = state.update(Action::Patched {
        ticket,
        deck_card: Box::new(state.entries[0].deck_card.clone()),
    });
    assert_eq!(
        undo_records(&effects),
        [UndoAction::PrintingChanged {
            old_card: Box::new(old),
            new_id: id(&new)
        }]
    );
    assert!(state.row(id(&new)).is_some());
    assert_eq!(state.printing().map(id), Some(id(&new)));
}

#[test]
fn a_zone_printing_patches_the_profile() {
    let commander = card("Boss");
    let (mut state, _, ticket, _) = loading(blank());
    let _ = state.update(Action::Loaded {
        ticket,
        deck: Box::new(deck(deck_profile("D"), Vec::new())),
        zone: Box::new(Zone {
            commander: Some(commander.clone()),
            ..Zone::default()
        }),
    });
    let new = card("Boss");
    let _ = state.update(Action::OpenPrintings {
        card: Box::new(commander.clone()),
        slot: Some(Slot::Commander),
    });
    let effects = state.update(Action::PickPrinting(Box::new(new.clone())));
    let [
        Effect::PatchProfile {
            ticket,
            op: Op::ChangePrinting,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("a zone printing patches the profile: {effects:?}");
    };
    let effects = state.update(Action::ProfilePatched { ticket: *ticket });
    assert_eq!(
        undo_records(&effects),
        [UndoAction::CommandZonePrintingChanged {
            slot: Slot::Commander,
            old_card: Box::new(commander),
            new_id: id(&new)
        }]
    );
    assert_eq!(state.featured()[0].0.scryfall_data.id, id(&new));
}

#[test]
fn undo_reverses_an_add() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let effects = state.update(Action::Undo(Box::new(UndoAction::Added {
        card_id: ring,
        card_name: "Sol Ring".to_owned(),
    })));
    assert!(matches!(
        effects.as_slice(),
        [
            Effect::Refilter,
            Effect::Toast(_),
            Effect::DeleteCard { op: Op::Undo, .. }
        ]
    ));
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Removed Sol Ring".to_owned())]
    );
    assert!(state.row(ring).is_none());
}

#[test]
fn undo_re_adds_a_removal_and_drops_it_on_failure() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let entry = state.entries[0].clone();
    let removal = state.update(Action::ChangeQuantity {
        card_id: ring,
        delta: -1,
    });
    assert!(state.row(ring).is_none());
    let effects = state.update(Action::Undo(Box::new(UndoAction::Removed {
        entry: Box::new(entry),
        baseline: 3,
    })));
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Re-added Sol Ring".to_owned())]
    );
    assert_eq!(state.quantity(ring), 3);
    let Some(Effect::CreateCard {
        ticket, request, ..
    }) = effects.last()
    else {
        panic!("undo re-creates the card");
    };
    assert_eq!(request.quantity, 3);
    let _ = state.update(Action::Failed { ticket: *ticket });
    assert!(state.row(ring).is_none());
    drop(removal);
}

#[test]
fn undo_of_a_quantity_taps_the_inverse_without_recording() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let effects = state.update(Action::Undo(Box::new(UndoAction::QuantityChanged {
        card_id: ring,
        card_name: "Sol Ring".to_owned(),
        net: -2,
        baseline: 3,
    })));
    assert_eq!(state.quantity(ring), 3);
    assert_eq!(
        toasts(&effects).last(),
        Some(&(ToastKind::Info, "Sol Ring back to \u{00d7}3".to_owned()))
    );
    let ticket = debounce(&effects);
    let effects = state.update(Action::Debounced { ticket });
    let [Effect::PatchCard { ticket, .. }] = effects.as_slice() else {
        panic!("the inverse writes");
    };
    let effects = state.update(Action::Patched {
        ticket: *ticket,
        deck_card: Box::new(state.entries[0].deck_card.clone()),
    });
    assert!(undo_records(&effects).is_empty(), "no ping-pong");
}

#[test]
fn undo_moves_back_without_its_own_move_toast() {
    let (mut state, cards) = loaded();
    let ring = id(&cards[0]);
    let effects = state.update(Action::Undo(Box::new(UndoAction::MovedBoard {
        card_id: ring,
        card_name: "Sol Ring".to_owned(),
        from: Board::Maybeboard,
    })));
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Sol Ring back to maybeboard".to_owned())]
    );
    assert_eq!(undo_records(&effects), []);
    assert_eq!(state.row(ring).map(|r| r.board), Some(Board::Maybeboard));
}

#[test]
fn undo_restores_a_printing() {
    let (mut state, cards) = loaded();
    let old = card("Sol Ring");
    let effects = state.update(Action::Undo(Box::new(UndoAction::PrintingChanged {
        old_card: Box::new(old.clone()),
        new_id: id(&cards[0]),
    })));
    assert_eq!(
        toasts(&effects),
        [(ToastKind::Info, "Sol Ring printing restored".to_owned())]
    );
    assert!(state.row(id(&old)).is_some());
    assert!(matches!(
        effects.last(),
        Some(Effect::PatchCard { op: Op::Undo, card_id, .. }) if *card_id == id(&cards[0])
    ));
}

#[test]
fn a_stale_undo_says_already_changed() {
    let (mut state, cards) = loaded();
    let gone = Uuid::new_v4();
    for action in [
        UndoAction::Added {
            card_id: gone,
            card_name: "Gone".to_owned(),
        },
        UndoAction::Removed {
            entry: Box::new(state.entries[0].clone()),
            baseline: 1,
        },
        UndoAction::QuantityChanged {
            card_id: id(&cards[0]),
            card_name: "Sol Ring".to_owned(),
            net: i32::MIN,
            baseline: 1,
        },
        UndoAction::CommandZonePrintingChanged {
            slot: Slot::Commander,
            old_card: Box::new(card("Boss")),
            new_id: gone,
        },
    ] {
        let effects = state.update(Action::Undo(Box::new(action)));
        assert_eq!(
            toasts(&effects),
            [(ToastKind::Info, ALREADY_CHANGED.to_owned())]
        );
        assert_eq!(effects.len(), 1);
    }
}

#[test]
fn the_action_bar_navigates() {
    let (mut state, _, _, _) = loading(blank());
    let deck_id = state.deck_id().unwrap();
    assert!(matches!(
        state.update(Action::Back).as_slice(),
        [Effect::Navigate(Destination::Previous)]
    ));
    assert!(matches!(
        state.update(Action::OpenAdd).as_slice(),
        [Effect::Navigate(Destination::AddCards(d))] if *d == deck_id
    ));
    assert!(matches!(
        state.update(Action::OpenRemove).as_slice(),
        [Effect::Navigate(Destination::RemoveCards(d))] if *d == deck_id
    ));
}

#[test]
fn copies_count_quantities() {
    let (mut state, cards) = loaded();
    let _ = state.update(Action::ChangeQuantity {
        card_id: id(&cards[0]),
        delta: 3,
    });
    assert_eq!(state.copies(&cards[..2]), 5);
    assert_eq!(state.copies(&[card("stranger")]), 1);
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Cards");
    assert_eq!(HINT_TITLE, "Browsing your deck");
    assert_eq!(QUICK_ADD, "Quick add");
    assert_eq!(QUICK_ADD_PLACEHOLDER, "Search a card to add");
    assert_eq!(SEARCHING, "Searching...");
    assert_eq!(NO_RESULTS, "No results");
    assert_eq!(GROUP_BY, "Group by");
    assert_eq!(BOARDS, "Boards");
    assert_eq!(SHOW, "Show");
    assert_eq!(
        (TOKENS, COMMAND_ZONE, ART),
        ("Tokens", "Command zone", "Art")
    );
    assert_eq!(NO_CARDS, "No cards");
    assert_eq!(
        (BACK, ADD, REMOVE, FILTER, UNDO, RESET_FILTER),
        ("Back", "Add", "Remove", "Filter", "Undo", "Reset filter")
    );
    assert_eq!(FILTER_ACTIVE, "Filter is active");
    assert_eq!(FILTER_RESET, "Filter reset");
    assert_eq!(CARD_REMOVED, "Card removed");
    assert_eq!(ALREADY_CHANGED, "Already changed");
    assert_eq!(MVP, "MVP");
    let labels: Vec<&str> = BOARD_CHIPS.into_iter().map(board_label).collect();
    assert_eq!(labels, ["Main", "Maybe", "Side"]);
    assert_eq!(added("Sol Ring"), "Added Sol Ring");
    assert_eq!(moved_to(Board::Maybeboard), "Moved to maybeboard");
    assert_eq!(group_title("Creature", 4), "Creature (4)");
    assert_eq!(group_title("", 4), "(4)");
    assert_eq!(lands_title(36), "Lands (36)");
    assert_eq!(tokens_title(2), "Tokens (2)");
    assert_eq!(maybeboard_title(3), "Maybeboard (3)");
    assert_eq!(sideboard_title(1), "Sideboard (1)");
    assert_eq!(
        (TOKENS_KEY, LANDS_KEY, MAYBEBOARD_KEY, SIDEBOARD_KEY),
        ("Tokens", "Lands", "Maybeboard", "Sideboard")
    );
    assert_eq!(
        [
            Op::ChangeQuantity,
            Op::MoveBoard,
            Op::ToggleMvp,
            Op::ChangePrinting,
            Op::Undo,
            Op::QuickAddCard
        ]
        .map(Op::name),
        [
            "change_quantity",
            "move_board",
            "toggle_mvp",
            "change_printing",
            "undo",
            "quick_add_card"
        ]
    );
}
