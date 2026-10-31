use Color::{Black, Blue, Green, Red, White};
use crossterm::event::KeyCode;
use zwipe_core::domain::user::models::hints::HINT_SWIPE_SELECT;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::commander_swipe::{self, Mode};
use crate::inbound::widgets::toast::Toast;
use crate::test_support::{card, create_opened_with, render, screen_text, user};

/// The app's copy for this hint, as it reads.
const HINT: &str = "Commanders you save land here\n• Swipe up while picking a commander saves it here\n• Swipe deals commanders: right starts a deck, up saves one\n• Quick add searches by name\n• Create deck starts a deck with that commander\n• Remove takes it off the list";

/// A commander named `name` in `colors`.
fn colored(name: &str, colors: &[Color]) -> Card {
    let mut card = card(name);
    card.scryfall_data.color_identity = colors.to_vec().into();
    card
}

/// Atraxa (WUBG), Krenko (R), Talrand (U) and Brago (WU), newest save first.
fn saved() -> Vec<Card> {
    vec![
        colored("Atraxa", &[White, Blue, Black, Green]),
        colored("Krenko", &[Red]),
        colored("Talrand", &[Blue]),
        colored("Brago", &[White, Blue]),
    ]
}

/// Home with the maybeboard open over it, the hints already seen, and nothing queued.
fn on_maybeboard(verified: bool) -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut me = user(verified);
    for key in [HINT_COMMANDER_MAYBEBOARD, HINT_SWIPE_SELECT] {
        me.hints_shown.insert(key.to_owned(), true);
    }
    harness.app.shared.user = Some(me);
    harness.open(open);
    harness.clear();
    harness
}

/// [`on_maybeboard`] with [`saved`] landed on the first ticket.
fn loaded() -> Harness {
    let mut harness = on_maybeboard(true);
    harness.apply(Update::Loaded {
        ticket: 1,
        cards: saved(),
    });
    harness
}

fn view(harness: &Harness) -> &MaybeboardView {
    match harness.screen() {
        Screen::CommanderMaybeboard(view) => view,
        other => panic!("not the maybeboard: {other:?}"),
    }
}

fn view_mut(harness: &mut Harness) -> &mut MaybeboardView {
    match harness.screen_mut() {
        Screen::CommanderMaybeboard(view) => view,
        _ => panic!("not the maybeboard"),
    }
}

fn press(harness: &mut Harness, codes: &[KeyCode]) {
    for &code in codes {
        harness.press(code);
    }
}

fn names(cards: &[&Card]) -> Vec<String> {
    cards.iter().map(|c| c.scryfall_data.name.clone()).collect()
}

fn shown(harness: &Harness) -> Vec<String> {
    names(&view(harness).filtered())
}

fn last_toast(harness: &Harness) -> Toast {
    harness.app.shared.toasts.items().last().unwrap().clone()
}

fn oracle(card: &Card) -> Uuid {
    card.scryfall_data.oracle_id.unwrap()
}

/// Types `text` into Quick add from the screen's keys.
fn type_query(harness: &mut Harness, text: &str) {
    harness.press(KeyCode::Char('/'));
    harness.type_text(text);
}

/// Puts the focus on the rows and expands the first.
fn expand_first(harness: &mut Harness) {
    view_mut(harness).focus = Focus::List;
    view_mut(harness).rows.set_focused(0);
    harness.press(KeyCode::Enter);
}

#[test]
fn the_maybeboard_opens_asking_for_everything_once() {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(open);
    assert!(matches!(harness.screen(), Screen::CommanderMaybeboard(_)));
    assert_eq!(
        harness.pending(),
        [
            Request::Load { ticket: 1 }.into(),
            Request::DeckProfiles.into(),
            oracle_tags::Request::Catalog.into(),
            screens::Request::MarkHintShown(HINT_COMMANDER_MAYBEBOARD.to_owned()),
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, "Commander maybeboard");
    assert_eq!(
        hint.content.text(),
        "Commanders you save land here\n• Swipe up while picking a commander saves it here\n• Swipe deals commanders: right starts a deck, up saves one\n• Quick add searches by name\n• Create deck starts a deck with that commander\n• Remove takes it off the list"
    );
}

#[test]
fn shift_h_shows_the_hint_again() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('H'));
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, TITLE);
    assert_eq!(hint.content.text(), HINT);
}

#[test]
fn the_skeleton_ghosts_everything() {
    let harness = on_maybeboard(true);
    let text = screen_text(&harness.app);
    assert!(text.contains('░'));
    assert!(text.contains("Commander maybeboard"), "the title is real");
    for real in [
        "Quick add",
        "Search a commander",
        "Show",
        "All",
        "Commanders ",
    ] {
        assert!(!text.contains(real), "{real} shows while loading");
    }
    assert!(text.contains("s Swipe"));
    assert!(text.contains("M More"));
}

#[test]
fn the_loaded_screen_draws_quick_add_show_and_the_rows() {
    let harness = loaded();
    let text = screen_text(&harness.app);
    for piece in [
        "Quick add",
        "Search a commander to add",
        "Show",
        "All",
        "Commanders",
        "Atraxa",
        "Krenko",
        "Talrand",
        "Brago",
        "Esc Back",
        "f Filter",
        "s Swipe",
        "M More",
    ] {
        assert!(text.contains(piece), "{piece} is missing");
    }
    assert!(!text.contains("Art"), "the terminal has no Art chip");
}

#[test]
fn an_empty_maybeboard_says_so() {
    let mut harness = on_maybeboard(true);
    harness.apply(Update::Loaded {
        ticket: 1,
        cards: Vec::new(),
    });
    let text = screen_text(&harness.app);
    assert!(text.contains(NO_COMMANDERS));
    assert!(text.contains("Show"), "the Show row stays");
    assert!(!text.contains("Commanders "));
}

#[test]
fn a_failed_first_load_says_so_and_toasts() {
    let mut harness = on_maybeboard(true);
    harness.apply(Update::LoadFailed {
        ticket: 1,
        message: "Server down".to_owned(),
    });
    assert!(view(&harness).load_failed);
    assert!(screen_text(&harness.app).contains(LOAD_FAILED));
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Error, "Server down")
    );
}

#[test]
fn a_failed_refetch_keeps_the_rows() {
    let mut harness = loaded();
    let mut cx = Context::new(&mut harness.app.shared);
    let Screen::CommanderMaybeboard(open) = harness.app.nav.top_mut() else {
        panic!("not the maybeboard");
    };
    open.refetch(&mut cx);
    harness.apply(Update::LoadFailed {
        ticket: 2,
        message: "Server down".to_owned(),
    });
    assert!(!view(&harness).load_failed);
    assert_eq!(shown(&harness).len(), 4);
}

#[test]
fn only_the_newest_load_lands() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('s'));
    harness.press(KeyCode::Esc);
    assert_eq!(view(&harness).ticket, 2);
    harness.apply(Update::Loaded {
        ticket: 1,
        cards: vec![colored("Old", &[])],
    });
    assert_eq!(shown(&harness).len(), 4, "a stale reply landed");
    harness.apply(Update::Loaded {
        ticket: 2,
        cards: vec![colored("Fresh", &[])],
    });
    assert_eq!(shown(&harness), ["Fresh"]);
}

#[test]
fn a_load_sent_before_a_remove_does_not_bring_the_row_back() {
    let mut harness = loaded();
    let atraxa = oracle(&saved_card(&harness, 0));
    harness.press(KeyCode::Char('s'));
    harness.press(KeyCode::Esc);
    harness.clear();
    let before = view(&harness).entries.clone();
    harness.apply(Update::Removed(atraxa));
    assert_eq!(
        harness.pending(),
        [Request::Load { ticket: 3 }.into()],
        "the load that predates the remove is not started again"
    );
    harness.apply(Update::Loaded {
        ticket: 2,
        cards: before,
    });
    assert_eq!(shown(&harness), ["Krenko", "Talrand", "Brago"]);
}

#[test]
fn a_remove_with_no_load_out_fetches_nothing() {
    let mut harness = loaded();
    let atraxa = oracle(&saved_card(&harness, 0));
    harness.apply(Update::Removed(atraxa));
    assert_eq!(harness.pending(), []);
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Info, REMOVED)
    );
}

fn saved_card(harness: &Harness, index: usize) -> Card {
    view(harness).entries[index].clone()
}

#[test]
fn r_on_an_expanded_row_removes_it() {
    let mut harness = loaded();
    expand_first(&mut harness);
    let text = screen_text(&harness.app);
    assert!(text.contains("c Create deck"));
    assert!(text.contains("r Remove"));
    harness.press(KeyCode::Char('r'));
    let atraxa = oracle(&saved_card(&harness, 0));
    assert_eq!(harness.pending(), [Request::Remove(atraxa).into()]);
    harness.apply(Update::Removed(atraxa));
    assert_eq!(shown(&harness), ["Krenko", "Talrand", "Brago"]);
    assert_eq!(last_toast(&harness).message, "Removed");
}

#[test]
fn c_on_an_expanded_row_starts_a_deck_behind_the_gate() {
    let mut harness = loaded();
    expand_first(&mut harness);
    harness.press(KeyCode::Char('c'));
    assert_eq!(create_opened_with(&harness), Some("Atraxa"));

    let mut harness = on_maybeboard(false);
    harness.apply(Update::Loaded {
        ticket: 1,
        cards: saved(),
    });
    harness.apply(Update::DeckProfiles(1));
    expand_first(&mut harness);
    harness.press(KeyCode::Char('c'));
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (
            ToastKind::Warning,
            "Verify your email to create more than 1 deck"
        )
    );
}

#[test]
fn the_show_pips_keep_cards_holding_every_picked_color() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Show;
    // W is the second chip.
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(view(&harness).colors, [White]);
    assert_eq!(shown(&harness), ["Atraxa", "Brago"]);
    // U too: both still hold white and blue.
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(shown(&harness), ["Atraxa", "Brago"]);
    // B narrows to Atraxa alone.
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(shown(&harness), ["Atraxa"]);
    // R matches nothing.
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(shown(&harness), [""; 0]);
    assert!(screen_text(&harness.app).contains(NO_MATCH));
    // Picking R again drops it.
    harness.press(KeyCode::Enter);
    assert_eq!(shown(&harness), ["Atraxa"]);
    // All clears every color.
    view_mut(&mut harness).show_cursor = 0;
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).colors, []);
    assert_eq!(shown(&harness).len(), 4);
}

#[test]
fn the_search_query_is_the_apps() {
    let mut want = CardQueryBuilder::with_name_contains("atr");
    want.set_is_commander_in_format(Format::Commander)
        .set_is_token(false)
        .set_sort(CardSortKey::Name)
        .set_limit(8);
    assert_eq!(search_query("  atr "), want);
}

#[test]
fn quick_add_waits_out_the_debounce_and_only_the_newest_query_searches() {
    let mut harness = loaded();
    type_query(&mut harness, "a");
    assert!(harness.pending().is_empty(), "one character never searches");
    assert!(!view(&harness).quick.searching);
    harness.type_text("t");
    assert_eq!(harness.pending(), [Request::Debounce { ticket: 2 }.into()]);
    assert!(screen_text(&harness.app).contains(SEARCHING));
    harness.type_text("r");
    harness.clear();
    harness.apply(Update::Debounced { ticket: 2 });
    assert!(
        harness.pending().is_empty(),
        "an older debounce searched after a newer edit"
    );
    harness.apply(Update::Debounced { ticket: 3 });
    assert_eq!(
        harness.pending(),
        [Request::Search {
            ticket: 3,
            query: "atr".to_owned(),
        }
        .into()]
    );
    harness.apply(Update::Searched {
        ticket: 2,
        cards: vec![colored("Stale", &[])],
    });
    assert!(view(&harness).quick.searching, "a stale result landed");
    harness.apply(Update::Searched {
        ticket: 3,
        cards: vec![
            colored("Atraxa, Grand Unifier", &[]),
            colored("Atarka", &[]),
        ],
    });
    assert!(!view(&harness).quick.searching);
    let text = screen_text(&harness.app);
    assert!(text.contains("Atraxa, Grand Unifier"));
    assert!(text.contains("Atarka"));
    assert!(!text.contains(SEARCHING));
}

#[test]
fn typed_letters_go_to_the_box_not_the_keys() {
    let mut harness = loaded();
    type_query(&mut harness, "sMf");
    assert!(matches!(harness.screen(), Screen::CommanderMaybeboard(_)));
    assert_eq!(view(&harness).quick.query.value(), "sMf");
    assert!(view(&harness).overlay.is_none());
    // Esc clears the query, as the app's "×", and hands the keys back.
    harness.press(KeyCode::Esc);
    assert!(view(&harness).quick.query.is_empty());
    assert!(!view(&harness).quick.typing);
    assert_eq!(view(&harness).quick.results, []);
    assert!(matches!(harness.screen(), Screen::CommanderMaybeboard(_)));
}

#[test]
fn x_on_the_box_clears_the_query() {
    let mut harness = loaded();
    type_query(&mut harness, "kr");
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains("×"));
    harness.press(KeyCode::Char('x'));
    assert!(view(&harness).quick.query.is_empty());
    assert!(!screen_text(&harness.app).contains("×"));
}

#[test]
fn a_failed_search_keeps_the_old_results() {
    let mut harness = loaded();
    type_query(&mut harness, "at");
    harness.apply(Update::Searched {
        ticket: 2,
        cards: vec![colored("Atarka", &[])],
    });
    harness.type_text("a");
    harness.apply(Update::SearchFailed { ticket: 3 });
    assert!(!view(&harness).quick.searching);
    assert_eq!(names(&view(&harness).shown_results()), ["Atarka"]);
}

#[test]
fn saved_commanders_never_show_as_results() {
    let mut harness = loaded();
    let krenko = saved_card(&harness, 1);
    type_query(&mut harness, "kr");
    harness.apply(Update::Searched {
        ticket: 2,
        cards: vec![krenko],
    });
    assert_eq!(names(&view(&harness).shown_results()), [""; 0]);
    assert!(screen_text(&harness.app).contains(NO_RESULTS));
}

#[test]
fn picking_a_result_drops_its_chip_adds_it_on_top_and_refetches() {
    let mut harness = loaded();
    type_query(&mut harness, "at");
    let atarka = colored("Atarka", &[Red, Green]);
    harness.apply(Update::Searched {
        ticket: 2,
        cards: vec![colored("Arcades", &[]), atarka.clone()],
    });
    // Enter hands the keys to the chips; l moves to the second.
    press(&mut harness, &[KeyCode::Enter, KeyCode::Char('l')]);
    assert_eq!(view(&harness).focus, Focus::Results);
    harness.clear();
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [Request::Add(Box::new(atarka.clone())).into()]
    );
    assert_eq!(names(&view(&harness).shown_results()), ["Arcades"]);
    harness.clear();
    harness.apply(Update::Added(Box::new(atarka.clone())));
    assert_eq!(shown(&harness).first().unwrap(), "Atarka");
    assert_eq!(harness.pending(), [Request::Load { ticket: 2 }.into()]);
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Info, "Added Atarka")
    );
    // A second confirmation never duplicates the row.
    harness.apply(Update::Added(Box::new(atarka)));
    assert_eq!(shown(&harness).len(), 5);
}

#[test]
fn clear_asks_first_then_empties_the_list() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('M'));
    assert!(matches!(view(&harness).overlay, Some(Overlay::More(_))));
    let text = screen_text(&harness.app);
    assert!(text.contains("More actions"));
    assert!(text.contains("Clear maybeboard"));
    harness.press(KeyCode::Char('c'));
    let text = screen_text(&harness.app);
    assert!(text.contains("Every commander on your maybeboard will be removed."));
    assert!(text.contains("Cancel"));
    // Cancel goes back to the sheet and asks for nothing.
    harness.press(KeyCode::Char('n'));
    assert!(matches!(view(&harness).overlay, Some(Overlay::More(_))));
    assert_eq!(harness.pending(), []);
    press(&mut harness, &[KeyCode::Char('c'), KeyCode::Char('y')]);
    assert!(view(&harness).overlay.is_none());
    assert_eq!(harness.pending(), [Request::Clear.into()]);
    harness.apply(Update::Cleared);
    assert_eq!(view(&harness).entries, []);
    assert_eq!(last_toast(&harness).message, "Maybeboard cleared");
    assert!(screen_text(&harness.app).contains(NO_COMMANDERS));
}

#[test]
fn esc_closes_the_more_sheet() {
    let mut harness = loaded();
    press(&mut harness, &[KeyCode::Char('M'), KeyCode::Esc]);
    assert!(view(&harness).overlay.is_none());
    assert!(matches!(harness.screen(), Screen::CommanderMaybeboard(_)));
}

#[test]
fn swipe_deals_commanders_without_the_saved_ones_and_refetches_on_close() {
    let mut harness = loaded();
    let want = view(&harness).saved();
    harness.press(KeyCode::Char('s'));
    let Screen::CommanderSwipe(swipe) = harness.screen() else {
        panic!("the swipe did not open");
    };
    let Mode::Commander(search) = &swipe.mode else {
        panic!("not the Commander mode");
    };
    assert_eq!(search.exclude, want);
    assert_eq!(
        harness.pending(),
        [commander_swipe::Request::Search {
            ticket: 0,
            offset: 0,
            filter: Box::default(),
        }
        .into()]
    );
    harness.clear();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::CommanderMaybeboard(_)));
    assert_eq!(harness.pending(), [Request::Load { ticket: 2 }.into()]);
    assert!(harness.app.shared.caches.handoff.commander_swipe.is_none());
}

#[test]
fn a_picked_printing_swaps_in_place() {
    let mut harness = loaded();
    expand_first(&mut harness);
    let mut printing = saved_card(&harness, 0);
    printing.scryfall_data.id = Uuid::new_v4();
    harness.app.shared.caches.handoff.printing = Some(printing.clone());
    let mut cx = Context::new(&mut harness.app.shared);
    let Screen::CommanderMaybeboard(view) = harness.app.nav.top_mut() else {
        panic!("not the maybeboard");
    };
    view.on_return(&mut cx);
    assert!(
        cx.into_effects().requests.is_empty(),
        "a printing refetches nothing"
    );
    assert_eq!(view.entries[0].scryfall_data.id, printing.scryfall_data.id);
    assert_eq!(view.rows.expanded(), Some(printing.scryfall_data.id));
}

#[test]
fn filter_opens_the_card_filter_and_the_list_narrows_only_on_apply() {
    use crate::domain::filter::sections::Section as FilterSection;

    let mut harness = loaded();
    harness.press(KeyCode::Char('f'));
    harness.app.shared.hint = None;
    let sheet = view(&harness)
        .sheet
        .as_ref()
        .expect("f opens the filter sheet");
    assert!(
        !sheet.sections().contains(&FilterSection::Format),
        "no format filter here"
    );
    assert!(screen_text(&harness.app).contains(" Filter "));

    if let Some(sheet) = &mut view_mut(&mut harness).sheet {
        sheet.draft.set_name_contains("ra");
        sheet.draft.set_sort(CardSortKey::Name);
    }
    assert_eq!(
        shown(&harness),
        ["Atraxa", "Krenko", "Talrand", "Brago"],
        "a draft does not reshuffle the list"
    );
    harness.press(KeyCode::Char('a'));
    assert!(view(&harness).sheet.is_none(), "Apply closes the sheet");
    assert_eq!(shown(&harness), ["Atraxa", "Brago", "Talrand"]);
    assert!(screen_text(&harness.app).contains("Filter •"));

    harness.press(KeyCode::Char('f'));
    if let Some(sheet) = &mut view_mut(&mut harness).sheet {
        sheet.draft = CardQueryBuilder::default();
    }
    harness.press(KeyCode::Char('a'));
    assert_eq!(shown(&harness), ["Atraxa", "Krenko", "Talrand", "Brago"]);
    assert!(!screen_text(&harness.app).contains("Filter •"));
}

#[test]
fn the_filters_dictionary_lands_in_the_sheet_and_refetches_nothing() {
    use crate::domain::filter::fields::MatchGroup;
    use crate::domain::filter::picker::PickerField;

    let mut harness = loaded();
    harness.press(KeyCode::Char('f'));
    harness.app.shared.hint = None;
    if let Some(sheet) = &mut view_mut(&mut harness).sheet {
        sheet.dictionary = Some(PickerField::Exclude(MatchGroup::Tags));
    }
    harness.open(oracle_tags::open);
    harness.app.shared.hint = None;
    harness.clear();
    harness.app.shared.caches.handoff.oracle_tag = Some("ramp".to_owned());
    harness.press(KeyCode::Esc);
    let sheet = view(&harness)
        .sheet
        .as_ref()
        .expect("the sheet is still open");
    assert_eq!(
        sheet.draft.oracle_tags_excludes(),
        Some(&["ramp".to_owned()][..])
    );
    assert_eq!(harness.pending(), []);
}

#[test]
fn esc_goes_back() {
    let mut harness = loaded();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn tiny_terminals_do_not_panic() {
    let mut harness = on_maybeboard(true);
    for (w, h) in [(8, 3), (1, 1)] {
        render(&harness.app, w, h);
    }
    harness.apply(Update::Loaded {
        ticket: 1,
        cards: saved(),
    });
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
    type_query(&mut harness, "at");
    harness.apply(Update::Searched {
        ticket: 2,
        cards: vec![colored("Atarka", &[])],
    });
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
    harness.press(KeyCode::Enter);
    expand_first(&mut harness);
    press(&mut harness, &[KeyCode::Char('M'), KeyCode::Char('c')]);
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
}

#[test]
fn the_maybeboard_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&on_maybeboard(true).app, "Esc Back");
    crate::test_support::assert_key_bar_fits(&loaded().app, "Esc Back");
}
