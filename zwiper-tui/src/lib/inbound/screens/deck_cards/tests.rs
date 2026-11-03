use crossterm::event::KeyCode;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Format, Quantity};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck, deck_profile, render, screen_text, user};

fn typed(name: &str, type_line: &str) -> Card {
    let mut c = card(name);
    c.scryfall_data.type_line = Some(type_line.to_owned());
    c
}

/// The view over a commander deck holding a creature, a land and a maybeboard card, loaded, after `edit` has had the cards.
fn loaded_with(edit: impl FnOnce(&mut [Card])) -> (Harness, Uuid, Vec<Card>) {
    let commander = typed("Boss", "Legendary Creature — Goblin");
    let mut cards = vec![
        typed("Elf", "Creature — Elf"),
        typed("Forest", "Basic Land — Forest"),
        typed("Maybe", "Instant"),
    ];
    edit(&mut cards);
    let mut profile = deck_profile("Gobbos");
    profile.format = Some(Format::Commander);
    profile.commander_id = Some(commander.scryfall_data.id);
    profile.land_target = Some(1);
    let deck_id = profile.id;
    let mut d = deck(profile, cards.clone());
    d.entries[2].deck_card.board = Board::Maybeboard;
    d.command_zone_cards = vec![commander.clone()];
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, deck_id));
    serve(
        &mut harness,
        d,
        Zone {
            commander: Some(commander),
            ..Zone::default()
        },
    );
    harness.clear();
    (harness, deck_id, cards)
}

fn loaded() -> (Harness, Uuid, Vec<Card>) {
    loaded_with(|_| {})
}

/// Answers the newest deck fetch with `deck` and `zone`.
fn serve(harness: &mut Harness, deck: Deck, zone: Zone) {
    let ticket = fetch_ticket(harness);
    harness.apply(Update::Loaded {
        ticket,
        profile: None,
        deck: Box::new(deck),
        zone: Box::new(zone),
    });
}

fn view(harness: &Harness) -> &DeckCardsView {
    match harness.screen() {
        Screen::DeckCards(view) => view,
        other => panic!("not on deck cards: {other:?}"),
    }
}

fn view_mut(harness: &mut Harness) -> &mut DeckCardsView {
    match harness.screen_mut() {
        Screen::DeckCards(view) => view,
        other => panic!("not on deck cards: {other:?}"),
    }
}

fn toasts(harness: &Harness) -> Vec<String> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.clone())
        .collect()
}

fn last_toast(harness: &Harness) -> String {
    toasts(harness).last().cloned().unwrap()
}

fn requests(harness: &Harness) -> Vec<&Request> {
    harness
        .pending()
        .iter()
        .filter_map(|r| match r {
            screens::Request::DeckCards(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// The newest deck fetch's ticket.
fn fetch_ticket(harness: &Harness) -> Ticket {
    requests(harness)
        .into_iter()
        .rev()
        .find_map(|r| match r {
            Request::FetchDeck { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no deck fetch went out: {:?}", harness.pending()))
}

/// The tickets of the waits that went out, oldest first.
fn debounces(harness: &Harness) -> Vec<Ticket> {
    requests(harness)
        .into_iter()
        .filter_map(|r| match r {
            Request::Debounce { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .collect()
}

/// The ticket of the one request queued, which must be a write or a search.
fn only_ticket(harness: &Harness) -> Ticket {
    match requests(harness)[..] {
        [
            Request::SearchCards { ticket, .. }
            | Request::CreateCard { ticket, .. }
            | Request::PatchCard { ticket, .. }
            | Request::DeleteCard { ticket, .. }
            | Request::PatchProfile { ticket, .. },
        ] => *ticket,
        ref other => panic!("not one request: {other:?}"),
    }
}

/// A deck card the server holds for `card`.
fn deck_card(deck_id: Uuid, card: &Card, quantity: i32) -> DeckCard {
    DeckCard {
        deck_id,
        scryfall_data_id: card.scryfall_data.id,
        oracle_id: card.scryfall_data.oracle_id.unwrap(),
        quantity: Quantity::new(quantity).unwrap(),
        board: Board::Deck,
        mvp_at: None,
    }
}

/// Moves the cursor onto the row for `name` and opens it.
fn open_row(harness: &mut Harness, name: &str) {
    let at = view(harness)
        .items()
        .iter()
        .position(|i| matches!(i, Item::Card { card, .. } if card.scryfall_data.name == name))
        .unwrap();
    view_mut(harness).cursor = at;
    harness.press(KeyCode::Enter);
    assert!(view(harness).open_row().is_some());
}

#[test]
fn opening_loads_the_deck_and_tokens() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let deck_id = Uuid::new_v4();
    harness.open(|cx| open(cx, deck_id));
    assert!(matches!(
        requests(&harness)[..],
        [
            Request::FetchDeck { deck_id: a, .. },
            Request::FetchTokens { deck_id: b, .. },
        ] if *a == deck_id && *b == deck_id
    ));
    assert!(screen_text(&harness.app).contains("? hint"));
}

#[test]
fn the_list_shows_the_zone_groups_and_lands_in_order() {
    let (harness, _, _) = loaded();
    let titles: Vec<String> = view(&harness)
        .items()
        .iter()
        .filter_map(|i| match i {
            Item::Header { title, .. } => Some(title.clone()),
            Item::Card { .. } => None,
        })
        .collect();
    assert_eq!(titles.first().map(String::as_str), Some("Commander"));
    assert_eq!(titles.last().map(String::as_str), Some("Lands (1)"));
    let screen = screen_text(&harness.app);
    for expected in [
        "Gobbos",
        "Commander",
        "Boss",
        "Elf",
        "Quick add",
        "Group by",
        "Boards",
        "Show",
        " Tokens ",
        " Command zone ",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    assert!(
        !screen.contains("Maybeboard ("),
        "the maybeboard is off by default"
    );
    assert!(
        !screen.contains(" Art "),
        "rows draw no thumbnails, so there is no Art chip"
    );
}

#[test]
fn a_remembered_filter_says_it_is_active_and_the_hint_shows_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    let d = deck(deck_profile("D"), vec![card("A")]);
    let deck_id = d.deck_profile.id;
    harness.app.shared.cards.filters.park(
        FilterScope::Cards,
        deck_id,
        CardQueryBuilder::with_name_contains("zzz"),
    );
    harness.open(|cx| open(cx, deck_id));
    serve(&mut harness, d, Zone::default());
    assert_eq!(last_toast(&harness), "Filter is active");
    assert_eq!(
        harness.app.shared.hint.as_ref().unwrap().content.title,
        "Browsing your deck"
    );
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains("No cards"));
    harness.press(KeyCode::Char('x'));
    assert_eq!(last_toast(&harness), "Filter reset");
    assert!(view(&harness).filter.is_empty());
    assert!(!screen_text(&harness.app).contains("No cards"));
}

#[test]
fn quantity_taps_gather_into_one_write_and_record_once_written() {
    let (mut harness, deck_id, cards) = loaded();
    open_row(&mut harness, "Elf");
    let id = cards[0].scryfall_data.id;
    harness.press(KeyCode::Char('+'));
    harness.press(KeyCode::Char('+'));
    assert_eq!(view(&harness).model.quantity(id), 3, "applied at once");
    let [first, second] = debounces(&harness)[..] else {
        panic!("one wait per tap");
    };
    harness.clear();
    harness.apply(Update::Debounced { ticket: first });
    assert!(
        requests(&harness).is_empty(),
        "an older tap's wait does nothing"
    );
    harness.apply(Update::Debounced { ticket: second });
    let [Request::PatchCard { body, .. }] = requests(&harness)[..] else {
        panic!("one write");
    };
    assert_eq!(*body, Body(Box::new(HttpPatchDeckCard::new(Some(3), None))));
    let ticket = only_ticket(&harness);
    harness.apply(Update::Patched {
        ticket,
        deck_card: Box::new(deck_card(deck_id, &cards[0], 3)),
    });
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 1);

    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Elf back to ×1");
    assert_eq!(view(&harness).model.quantity(id), 1);
}

#[test]
fn minus_at_one_removes_the_row_and_a_failed_delete_puts_it_back() {
    let (mut harness, deck_id, cards) = loaded();
    open_row(&mut harness, "Forest");
    let id = cards[1].scryfall_data.id;
    harness.press(KeyCode::Char('-'));
    let messages = toasts(&harness);
    assert!(messages.contains(&"Card removed".to_owned()));
    assert!(messages.contains(&"Below land target (1)".to_owned()));
    assert!(view(&harness).model.row(id).is_none());
    let [wait] = debounces(&harness)[..] else {
        panic!("one wait");
    };
    harness.clear();
    harness.apply(Update::Debounced { ticket: wait });
    assert!(matches!(
        requests(&harness)[..],
        [Request::DeleteCard { card_id, .. }] if *card_id == id
    ));
    let ticket = only_ticket(&harness);
    harness.apply(Update::Failed {
        ticket,
        message: Some("offline".to_owned()),
    });
    assert_eq!(last_toast(&harness), "offline");
    assert_eq!(view(&harness).model.quantity(id), 1, "the row is back");
    assert!(view(&harness).model.row(id).is_some());
    assert!(
        harness.app.shared.cards.undo.is_empty(deck_id),
        "its undo entry is dropped"
    );
}

#[test]
fn moves_stars_and_undo_reverse_in_order() {
    let (mut harness, deck_id, cards) = loaded();
    open_row(&mut harness, "Elf");
    let id = cards[0].scryfall_data.id;
    harness.press(KeyCode::Char('s'));
    assert!(view(&harness).model.row(id).unwrap().mvp);
    assert!(screen_text(&harness.app).contains("MVP"));
    harness.clear();
    harness.press(KeyCode::Char('b'));
    assert_eq!(last_toast(&harness), "Moved to sideboard");
    assert_eq!(view(&harness).board(id), Some(Board::Sideboard));
    let ticket = only_ticket(&harness);
    harness.apply(Update::Failed {
        ticket,
        message: Some("nope".to_owned()),
    });
    assert_eq!(view(&harness).board(id), Some(Board::Deck));
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Elf back to deck");
    harness.press(KeyCode::Char('u'));
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 0);
}

#[test]
fn a_stale_undo_says_already_changed() {
    let (mut harness, deck_id, _) = loaded();
    harness.app.shared.cards.undo.push(
        deck_id,
        UndoAction::Added {
            card_id: Uuid::new_v4(),
            card_name: "Gone".to_owned(),
        },
    );
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Already changed");
}

#[test]
fn quick_add_waits_searches_and_adds() {
    let (mut harness, deck_id, _) = loaded();
    harness.press(KeyCode::Char('/'));
    harness.type_text("g");
    assert_eq!(view(&harness).model.quick_add(), None, "one letter waits");
    harness.type_text("o");
    assert_eq!(
        view(&harness).model.quick_add(),
        Some(QuickAddResults::Searching)
    );
    let [wait] = debounces(&harness)[..] else {
        panic!("one wait");
    };
    harness.clear();
    harness.apply(Update::Debounced { ticket: wait });
    assert!(matches!(
        requests(&harness)[..],
        [Request::SearchCards { .. }]
    ));
    let goblin = card("Goblin Guide");
    harness.apply(Update::Searched {
        ticket: only_ticket(&harness),
        cards: vec![goblin.clone()],
    });
    assert!(screen_text(&harness.app).contains("Goblin Guide"));
    harness.clear();
    harness.press(KeyCode::Enter);
    assert!(matches!(
        requests(&harness)[..],
        [Request::CreateCard {
            op: Op::QuickAddCard,
            ..
        }]
    ));
    harness.apply(Update::Created {
        ticket: only_ticket(&harness),
        deck_card: Box::new(deck_card(deck_id, &goblin, 1)),
    });
    assert_eq!(last_toast(&harness), "Added Goblin Guide");
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Removed Goblin Guide");
}

#[test]
fn a_late_or_repeated_quick_add_reply_changes_nothing() {
    let (mut harness, _, _) = loaded();
    harness.press(KeyCode::Char('/'));
    harness.type_text("go");
    let [first] = debounces(&harness)[..] else {
        panic!("one wait");
    };
    harness.clear();
    harness.apply(Update::Debounced { ticket: first });
    let search = only_ticket(&harness);
    harness.clear();
    harness.type_text("b");
    harness.apply(Update::Searched {
        ticket: search,
        cards: vec![card("Goblin Guide")],
    });
    assert_eq!(
        view(&harness).model.quick_add(),
        Some(QuickAddResults::Searching),
        "a search for an older query does not land"
    );
    harness.apply(Update::Debounced { ticket: first });
    assert!(
        requests(&harness)
            .iter()
            .all(|r| !matches!(r, Request::SearchCards { .. })),
        "an older pause sends no search"
    );
}

#[test]
fn a_late_or_repeated_deck_reply_changes_nothing() {
    let gobbos = deck(deck_profile("Gobbos"), vec![card("Elf")]);
    let deck_id = gobbos.deck_profile.id;
    let mut stranger = deck(deck_profile("Other"), vec![card("Stranger")]);
    stranger.deck_profile.id = deck_id;
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, deck_id));
    let first = fetch_ticket(&harness);
    serve(&mut harness, gobbos, Zone::default());
    let reply = |ticket| Update::Loaded {
        ticket,
        profile: None,
        deck: Box::new(stranger.clone()),
        zone: Box::new(Zone::default()),
    };
    harness.apply(reply(first));
    assert!(
        !screen_text(&harness.app).contains("Stranger"),
        "a repeated reply changes nothing"
    );

    harness.press(KeyCode::Char('r'));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::DeckCards(_)));
    let second = fetch_ticket(&harness);
    assert_ne!(first, second);
    harness.apply(reply(first));
    assert!(
        !view(&harness).model.is_loaded(),
        "a reply to the load before the return does not land"
    );
    let mut fresh = deck(deck_profile("Gobbos"), vec![card("Elf"), card("Orc")]);
    fresh.deck_profile.id = deck_id;
    serve(&mut harness, fresh, Zone::default());
    let text = screen_text(&harness.app);
    assert!(text.contains("Orc") && !text.contains("Stranger"), "{text}");
}

#[test]
fn add_writes_waiting_taps_and_its_return_loads_the_deck_again() {
    let (mut harness, _, cards) = loaded();
    open_row(&mut harness, "Elf");
    harness.press(KeyCode::Char('+'));
    harness.clear();
    harness.press(KeyCode::Char('a'));
    assert!(matches!(harness.screen(), Screen::Swipe(_)));
    assert!(requests(&harness).iter().any(|r| matches!(
        r,
        Request::Flush { card_id, write: Write::Set(2), .. } if *card_id == cards[0].scryfall_data.id
    )));
    harness.clear();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::DeckCards(_)));
    assert!(
        requests(&harness)
            .iter()
            .any(|r| matches!(r, Request::FetchDeck { .. })),
        "Add's return loads the deck again"
    );
}

#[test]
fn group_by_boards_and_show_toggle() {
    let (mut harness, _, _) = loaded();
    harness.press(KeyCode::Char('g'));
    assert_eq!(view(&harness).model.group_by(), GroupByOption::Cmc);
    harness.press(KeyCode::Char('3'));
    assert!(
        !view(&harness).model.board_on(Board::Sideboard),
        "no sideboard cards to show"
    );
    harness.press(KeyCode::Char('2'));
    assert!(screen_text(&harness.app).contains("Maybeboard (1)"));
    harness.press(KeyCode::Char('1'));
    harness.press(KeyCode::Char('2'));
    assert!(
        view(&harness).model.board_on(Board::Deck),
        "the last board snaps back to Main"
    );
    harness.press(KeyCode::Char('C'));
    assert!(!screen_text(&harness.app).contains("Commander ("));
    view_mut(&mut harness).cursor = 0;
    harness.press(KeyCode::Enter);
    let first = view(&harness).items().into_iter().next();
    assert!(
        matches!(&first, Some(Item::Header { key, .. }) if view(&harness).model.is_collapsed(key))
            || view(&harness).expanded.is_some()
    );
}

#[test]
fn the_filter_and_printing_sheets_open_and_esc_leaves() {
    let (mut harness, deck_id, cards) = loaded();
    harness.press(KeyCode::Char('f'));
    assert!(view(&harness).sheet.is_some());
    harness.press(KeyCode::Esc);
    assert!(view(&harness).sheet.is_none());
    open_row(&mut harness, "Elf");
    harness.press(KeyCode::Char('p'));
    assert!(view(&harness).printing.is_some());
    let mut other = cards[0].clone();
    other.scryfall_data.id = Uuid::new_v4();
    harness.apply(screens::card_sheets::Update::Printings {
        oracle_id: cards[0].scryfall_data.oracle_id.unwrap(),
        printings: Ok(vec![cards[0].clone(), other.clone()]),
    });
    harness.clear();
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('s'));
    assert_eq!(last_toast(&harness), "Printing saved");
    assert!(matches!(
        requests(&harness)[..],
        [Request::PatchCard {
            op: Op::ChangePrinting,
            ..
        }]
    ));
    harness.apply(Update::Patched {
        ticket: only_ticket(&harness),
        deck_card: Box::new(deck_card(deck_id, &other, 1)),
    });
    assert!(view(&harness).model.row(other.scryfall_data.id).is_some());
    assert_eq!(
        view(&harness).expanded,
        Some(other.scryfall_data.id),
        "the open row stays open on its new printing"
    );
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Elf printing restored");
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn the_filter_sheets_dictionary_returns_to_the_sheet_without_a_reload() {
    use zwipe_ui::filter::fields::MatchGroup;
    use zwipe_ui::filter::picker::PickerField;

    let (mut harness, _, _) = loaded();
    harness.press(KeyCode::Char('f'));
    harness.app.shared.hint = None;
    if let Some(sheet) = &mut view_mut(&mut harness).sheet {
        sheet.dictionary = Some(PickerField::Include(MatchGroup::Tags));
    }
    harness.open(screens::oracle_tags::open);
    assert!(matches!(harness.screen(), Screen::OracleTags(_)));
    harness.app.shared.hint = None;
    harness.clear();
    harness.app.shared.caches.handoff.oracle_tag = Some("ramp".to_owned());
    harness.press(KeyCode::Esc);
    let sheet = view(&harness)
        .sheet
        .as_ref()
        .expect("the sheet is still open");
    assert_eq!(
        sheet.filter.draft.oracle_tags_contains_any(),
        Some(&["ramp".to_owned()][..])
    );
    assert!(
        harness.pending().is_empty(),
        "no reload on the dictionary's return"
    );
}

#[test]
fn an_open_rows_tags_reveal_with_tab_and_x_opens_their_examples() {
    let (mut harness, _, _) = loaded_with(|cards| {
        let profile = &mut cards[0].card_profile;
        profile.card_roles.clear();
        profile.oracle_tags_by_role.clear();
        profile.other_oracle_tags = vec!["mana-dork".to_owned()];
    });
    open_row(&mut harness, "Elf");
    assert!(screen_text(&harness.app).contains("Tab tags"));
    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).open_tag, Some(0));
    assert!(screen_text(&harness.app).contains("No description yet"));
    harness.press(KeyCode::Char('x'));
    assert!(matches!(harness.screen(), Screen::OtagExamples(v) if v.model.slug() == "mana-dork"));
    harness.clear();
    harness.press(KeyCode::Esc);
    assert!(
        requests(&harness).is_empty(),
        "the examples' return keeps the view as it was"
    );
}

#[test]
fn the_view_fits_a_tiny_terminal() {
    let (mut harness, _, _) = loaded();
    render(&harness.app, 12, 5);
    open_row(&mut harness, "Elf");
    render(&harness.app, 12, 5);
    harness.press(KeyCode::Char('i'));
    render(&harness.app, 12, 5);
}

#[test]
fn the_view_keeps_every_key_on_a_narrow_terminal() {
    let (mut harness, _, _) = loaded();
    crate::test_support::assert_key_bar_fits(&harness.app, "? hint");
    open_row(&mut harness, "Elf");
    crate::test_support::assert_key_bar_fits(&harness.app, "? hint");
    for (width, height) in crate::test_support::NARROW {
        let screen = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            screen.contains("b to sideboard") && screen.contains("Elf"),
            "the open row keeps its name and every action in view at {width}x{height}:\n{screen}"
        );
    }
}
