use crossterm::event::KeyCode;
use zwipe_core::domain::deck::Format;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck, deck_profile, render, screen_text, user};

fn typed(name: &str, type_line: &str) -> Card {
    let mut c = card(name);
    c.scryfall_data.type_line = Some(type_line.to_owned());
    c
}

/// The view over a commander deck holding a creature, a land and a maybeboard card, loaded.
fn loaded() -> (Harness, Uuid, Vec<Card>) {
    let commander = typed("Boss", "Legendary Creature — Goblin");
    let cards = vec![
        typed("Elf", "Creature — Elf"),
        typed("Forest", "Basic Land — Forest"),
        typed("Maybe", "Instant"),
    ];
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
    harness.apply(Update::Loaded {
        deck_id,
        loaded: Box::new(Loaded {
            deck: d,
            profile: None,
            zone: Zone {
                commander: Some(commander),
                ..Zone::default()
            },
        }),
    });
    harness.clear();
    (harness, deck_id, cards)
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

fn last_toast(harness: &Harness) -> String {
    harness
        .app
        .shared
        .toasts
        .items()
        .last()
        .unwrap()
        .message
        .clone()
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
    assert_eq!(
        requests(&harness),
        [&Request::Load { deck_id }, &Request::Tokens { deck_id }]
    );
    assert!(screen_text(&harness.app).contains("Deck cards"));
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
    harness.apply(Update::Loaded {
        deck_id,
        loaded: Box::new(Loaded {
            deck: d,
            profile: None,
            zone: Zone::default(),
        }),
    });
    assert_eq!(last_toast(&harness), "Filter is active");
    assert_eq!(
        harness.app.shared.hint.as_ref().unwrap().content.title,
        "Browsing your deck"
    );
    assert!(screen_text(&harness.app).contains("No cards") || harness.app.shared.hint.is_some());
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Char('x'));
    assert_eq!(last_toast(&harness), "Filter reset");
    assert!(view(&harness).filter.is_empty());
}

#[test]
fn quantity_taps_gather_into_one_write_and_record_once_written() {
    let (mut harness, deck_id, cards) = loaded();
    open_row(&mut harness, "Elf");
    let id = cards[0].scryfall_data.id;
    harness.press(KeyCode::Char('+'));
    harness.press(KeyCode::Char('+'));
    assert_eq!(view(&harness).quantity(id), 3, "applied at once");
    let timers: Vec<u64> = requests(&harness)
        .iter()
        .filter_map(|r| match r {
            Request::FlushTimer { generation, .. } => Some(*generation),
            _ => None,
        })
        .collect();
    assert_eq!(timers, [1, 2]);
    harness.clear();
    harness.apply(Update::FlushDue {
        deck_id,
        card_id: id,
        generation: 1,
    });
    assert_eq!(
        requests(&harness).len(),
        0,
        "an older tap's timer does nothing"
    );
    harness.apply(Update::FlushDue {
        deck_id,
        card_id: id,
        generation: 2,
    });
    let [Request::WriteQuantity { burst, .. }] = requests(&harness)[..] else {
        panic!("one write");
    };
    assert_eq!(burst.write(), Write::Set(3));
    let burst = *burst;
    harness.apply(Update::QuantityWritten {
        deck_id,
        card_id: id,
        card_name: "Elf".to_owned(),
        burst,
    });
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 1);

    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Elf back to ×1");
    assert_eq!(view(&harness).quantity(id), 1);
}

#[test]
fn minus_at_one_removes_the_row_and_a_failed_delete_puts_it_back() {
    let (mut harness, deck_id, cards) = loaded();
    open_row(&mut harness, "Forest");
    let id = cards[1].scryfall_data.id;
    harness.press(KeyCode::Char('-'));
    let messages: Vec<String> = harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.clone())
        .collect();
    assert!(messages.contains(&"Card removed".to_owned()));
    assert!(messages.contains(&"Below land target (1)".to_owned()));
    assert!(view(&harness).entry(id).is_none());
    harness.clear();
    harness.apply(Update::FlushDue {
        deck_id,
        card_id: id,
        generation: 1,
    });
    let [Request::WriteQuantity { burst, removed, .. }] = requests(&harness)[..] else {
        panic!("one write");
    };
    let (burst, removed) = (*burst, removed.clone());
    assert_eq!(burst.write(), Write::Delete);
    harness.apply(Update::QuantityFailed {
        deck_id,
        card_id: id,
        burst,
        removed,
        message: "offline".to_owned(),
    });
    assert_eq!(last_toast(&harness), "offline");
    assert_eq!(view(&harness).quantity(id), 1, "the row is back");
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
    assert!(view(&harness).entry(id).unwrap().deck_card.mvp_at.is_some());
    assert!(screen_text(&harness.app).contains("MVP"));
    harness.press(KeyCode::Char('b'));
    assert_eq!(last_toast(&harness), "Moved to sideboard");
    assert_eq!(
        view(&harness).entry(id).unwrap().deck_card.board,
        Board::Sideboard
    );
    harness.apply(Update::MoveFailed {
        deck_id,
        card_id: id,
        from: Board::Deck,
        message: "nope".to_owned(),
    });
    assert_eq!(
        view(&harness).entry(id).unwrap().deck_card.board,
        Board::Deck
    );
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
    assert_eq!(
        view(&harness).quick.results,
        Dropdown::Closed,
        "one letter waits"
    );
    harness.type_text("o");
    assert_eq!(view(&harness).quick.results, Dropdown::Searching);
    let generation = view(&harness).quick.generation;
    harness.clear();
    harness.apply(Update::QuickDue {
        deck_id,
        generation,
    });
    assert!(matches!(
        requests(&harness)[..],
        [Request::QuickSearch { .. }]
    ));
    let goblin = card("Goblin Guide");
    harness.apply(Update::QuickResults {
        deck_id,
        generation,
        cards: Some(vec![goblin.clone()]),
    });
    assert!(screen_text(&harness.app).contains("Goblin Guide"));
    harness.clear();
    harness.press(KeyCode::Enter);
    assert!(matches!(requests(&harness)[..], [Request::QuickAdd { .. }]));
    let deck_card = DeckCard {
        deck_id,
        scryfall_data_id: goblin.scryfall_data.id,
        oracle_id: goblin.scryfall_data.oracle_id.unwrap(),
        quantity: Quantity::new(1).unwrap(),
        board: Board::Deck,
        mvp_at: None,
    };
    harness.apply(Update::QuickAdded {
        deck_id,
        entry: Box::new(DeckEntry {
            card: goblin,
            deck_card,
        }),
    });
    assert_eq!(last_toast(&harness), "Added Goblin Guide");
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Removed Goblin Guide");
}

#[test]
fn group_by_boards_and_show_toggle() {
    let (mut harness, _, _) = loaded();
    harness.press(KeyCode::Char('g'));
    assert_eq!(view(&harness).group_by, GroupByOption::Cmc);
    harness.press(KeyCode::Char('3'));
    assert!(!view(&harness).boards.side, "no sideboard cards to show");
    harness.press(KeyCode::Char('2'));
    assert!(screen_text(&harness.app).contains("Maybeboard (1)"));
    harness.press(KeyCode::Char('1'));
    harness.press(KeyCode::Char('2'));
    assert!(
        view(&harness).boards.main,
        "the last board snaps back to Main"
    );
    harness.press(KeyCode::Char('C'));
    assert!(!screen_text(&harness.app).contains("Commander ("));
    view_mut(&mut harness).cursor = 0;
    harness.press(KeyCode::Enter);
    assert!(!view(&harness).collapsed.is_empty() || view(&harness).expanded.is_some());
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
    assert!(matches!(requests(&harness)[..], [Request::Printing { .. }]));
    harness.apply(Update::PrintingSaved {
        deck_id,
        old_card: Box::new(cards[0].clone()),
        new_card: Box::new(other.clone()),
    });
    assert!(view(&harness).entry(other.scryfall_data.id).is_some());
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Elf printing restored");
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn the_filter_sheets_dictionary_returns_to_the_sheet_without_a_reload() {
    use crate::domain::filter::fields::MatchGroup;
    use crate::domain::filter::picker::PickerField;

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
        sheet.draft.oracle_tags_contains_any(),
        Some(&["ramp".to_owned()][..])
    );
    assert!(
        harness.pending().is_empty(),
        "no reload on the dictionary's return"
    );
}

#[test]
fn an_open_rows_tags_reveal_with_tab_and_x_opens_their_examples() {
    let (mut harness, _, _) = loaded();
    if let Some(entry) = view_mut(&mut harness)
        .entries
        .iter_mut()
        .find(|e| e.card.scryfall_data.name == "Elf")
    {
        let profile = &mut entry.card.card_profile;
        profile.card_roles.clear();
        profile.oracle_tags_by_role.clear();
        profile.other_oracle_tags = vec!["mana-dork".to_owned()];
    }
    view_mut(&mut harness).rerun_collapsed();
    open_row(&mut harness, "Elf");
    assert!(screen_text(&harness.app).contains("Tab tags"));
    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).open_tag, Some(0));
    assert!(screen_text(&harness.app).contains("No description yet"));
    harness.press(KeyCode::Char('x'));
    assert!(matches!(harness.screen(), Screen::OtagExamples(v) if v.slug == "mana-dork"));
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
