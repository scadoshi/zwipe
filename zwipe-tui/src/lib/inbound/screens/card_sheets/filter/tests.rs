use crossterm::event::KeyCode;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_type::CardType;

use super::*;
use crate::domain::filter::fields::{MatchGroup, Toggle};
use crate::domain::filter::sections::Row;
use crate::domain::stores::catalog::Kind;
use crate::domain::theme::Palette;
use crate::inbound::app::App;
use crate::inbound::screens;
use crate::inbound::widgets::hint::HintTopic;
use crate::test_support::{card, user};

fn app() -> App {
    App::new(Palette::terminal())
}

fn toasts(app: &App) -> Vec<(ToastKind, String)> {
    app.shared
        .toasts
        .items()
        .iter()
        .map(|t| (t.kind, t.message.clone()))
        .collect()
}

fn last_toast(app: &App) -> Option<(ToastKind, String)> {
    toasts(app).last().cloned()
}

/// Presses `codes` on `sheet`, returning the last outcome.
fn press(app: &mut App, sheet: &mut FilterSheet, codes: &[KeyCode]) -> SheetOutcome {
    let mut outcome = SheetOutcome::Open;
    for &code in codes {
        let mut cx = Context::new(&mut app.shared);
        outcome = keys::handle(sheet, code, &mut cx);
    }
    outcome
}

fn type_text(app: &mut App, sheet: &mut FilterSheet, text: &str) {
    let codes: Vec<KeyCode> = text.chars().map(KeyCode::Char).collect();
    press(app, sheet, &codes);
}

fn plain() -> SheetConfig {
    SheetConfig {
        show_format: true,
        ..SheetConfig::default()
    }
}

/// Focuses the header of `section` and opens it.
fn open_section(app: &mut App, sheet: &mut FilterSheet, section: Section) {
    sheet.focus = sheet
        .targets()
        .iter()
        .position(|t| *t == Target::Header(section))
        .unwrap();
    press(app, sheet, &[KeyCode::Enter]);
    assert_eq!(sheet.open, Some(section));
}

#[test]
fn opening_shows_the_hint_once_and_fetches_the_lists_this_host_needs() {
    let mut app = app();
    app.shared.user = Some(user(true));
    let mut cx = Context::new(&mut app.shared);
    FilterSheet::open(&mut cx, &CardQueryBuilder::default(), plain());
    let effects = cx.into_effects();
    assert_eq!(effects.requests.len(), 8, "the hint mark and seven lists");
    assert!(
        effects
            .requests
            .contains(&screens::Request::MarkHintShown("filter".to_owned()))
    );
    assert_eq!(app.shared.hint.as_ref().unwrap().content.title, "Filters");

    app.shared.hint = None;
    app.shared.cards.catalog.keywords.settle(None);
    let mut cx = Context::new(&mut app.shared);
    let config = SheetConfig {
        sources: Sources::deck(&[card("A")]),
        ..plain()
    };
    FilterSheet::open(&mut cx, &CardQueryBuilder::default(), config);
    let effects = cx.into_effects();
    assert!(app.shared.hint.is_none(), "the hint shows once");
    assert!(
        effects.requests.is_empty(),
        "a deck host needs only roles and tags, already loading"
    );
    assert_eq!(
        last_toast(&app),
        Some((
            ToastKind::Warning,
            "Filter lists didn't load, trying again".to_owned()
        ))
    );
}

#[test]
fn typing_a_name_then_applying_commits_the_draft() {
    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::Name);
    press(&mut app, &mut sheet, &[KeyCode::Down]);
    assert!(!sheet.typing(), "the label first");
    press(&mut app, &mut sheet, &[KeyCode::Down]);
    assert!(sheet.typing());
    type_text(&mut app, &mut sheet, "bolt");
    assert_eq!(sheet.draft.name_contains(), Some("bolt"));
    assert_eq!(
        press(&mut app, &mut sheet, &[KeyCode::Char('a')]),
        SheetOutcome::Open,
        "a types while in the box"
    );
    assert_eq!(sheet.draft.name_contains(), Some("bolta"));
    press(&mut app, &mut sheet, &[KeyCode::Backspace, KeyCode::Esc]);
    assert!(!sheet.typing(), "Esc leaves the box for the header");
    let outcome = press(&mut app, &mut sheet, &[KeyCode::Char('a')]);
    let SheetOutcome::Applied(filter) = outcome else {
        panic!("not applied: {outcome:?}");
    };
    assert_eq!(filter.name_contains(), Some("bolt"));
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Success, "Filter applied".to_owned()))
    );
}

#[test]
fn back_drops_edits_and_says_so_only_when_there_were_some() {
    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    assert_eq!(
        press(&mut app, &mut sheet, &[KeyCode::Esc]),
        SheetOutcome::Closed
    );
    assert_eq!(toasts(&app).len(), 0);

    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::Sort);
    press(
        &mut app,
        &mut sheet,
        &[KeyCode::Down, KeyCode::Down, KeyCode::Enter],
    );
    assert_eq!(sheet.draft.sort(), Some(CardSortKey::Name));
    assert_eq!(
        press(&mut app, &mut sheet, &[KeyCode::Esc]),
        SheetOutcome::Closed
    );
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Info, "Filter unchanged".to_owned()))
    );
}

#[test]
fn apply_refuses_an_empty_search_closes_on_no_change_and_names_contradictions() {
    let mut app = app();
    let validate = SheetConfig {
        validate: true,
        ..plain()
    };
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), validate);
    assert_eq!(
        press(&mut app, &mut sheet, &[KeyCode::Char('a')]),
        SheetOutcome::Open
    );
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Warning, "Filter is already empty".to_owned()))
    );

    let mut filter = CardQueryBuilder::default();
    filter.set_name_contains("elf");
    let mut sheet = FilterSheet::new(&filter, plain());
    assert_eq!(
        press(&mut app, &mut sheet, &[KeyCode::Char('a')]),
        SheetOutcome::Closed
    );

    sheet.draft.set_card_type_contains_any(vec![CardType::Land]);
    sheet.draft.set_card_type_excludes_any(vec![CardType::Land]);
    assert_eq!(
        press(&mut app, &mut sheet, &[KeyCode::Char('a')]),
        SheetOutcome::Open
    );
    assert_eq!(
        last_toast(&app),
        Some((
            ToastKind::Warning,
            "Filter can't both include and exclude card types".to_owned()
        ))
    );

    let mut sheet = FilterSheet::new(&filter, validate_config());
    sheet.draft.clear();
    let outcome = press(&mut app, &mut sheet, &[KeyCode::Char('a')]);
    assert!(
        matches!(outcome, SheetOutcome::Applied(_)),
        "a deliberate clear goes through"
    );
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Success, "Filter cleared".to_owned()))
    );
}

fn validate_config() -> SheetConfig {
    SheetConfig {
        validate: true,
        ..plain()
    }
}

#[test]
fn reset_follows_the_hosts_rule() {
    let mut app = app();
    let mut filter = CardQueryBuilder::default();
    filter.set_name_contains("elf");
    let mut sheet = FilterSheet::new(&filter, plain());
    press(&mut app, &mut sheet, &[KeyCode::Char('r')]);
    assert!(sheet.draft.is_empty());
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Info, "Filter reset".to_owned()))
    );

    let config = SheetConfig {
        reset: ResetRule::AddSearch(AddContext::default()),
        ..plain()
    };
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), config);
    press(&mut app, &mut sheet, &[KeyCode::Char('r')]);
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Warning, "Filter already at default".to_owned()))
    );
}

#[test]
fn chips_toggle_and_the_section_x_clears() {
    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::Types);
    press(&mut app, &mut sheet, &[KeyCode::Down, KeyCode::Down]);
    assert_eq!(
        sheet.focused(),
        Some(Target::Row(Row::Chips(ChipSet::BasicIncludes)))
    );
    press(
        &mut app,
        &mut sheet,
        &[KeyCode::Char('l'), KeyCode::Char(' ')],
    );
    assert_eq!(
        sheet.draft.card_type_contains_any(),
        Some(&[CardType::Creature][..])
    );
    assert!(Section::Types.is_active(&sheet.draft));
    press(
        &mut app,
        &mut sheet,
        &[KeyCode::Up, KeyCode::Up, KeyCode::Enter],
    );
    assert_eq!(sheet.open, None, "Enter on the open header closes it");
    press(&mut app, &mut sheet, &[KeyCode::Char('x')]);
    assert!(!Section::Types.is_active(&sheet.draft));
}

#[test]
fn numbers_parse_when_the_box_is_left() {
    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::Mana);
    press(&mut app, &mut sheet, &[KeyCode::Down, KeyCode::Down]);
    assert_eq!(
        sheet.focused(),
        Some(Target::Row(Row::Number(NumberField::ManaValue)))
    );
    type_text(&mut app, &mut sheet, "x");
    press(&mut app, &mut sheet, &[KeyCode::Enter]);
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Error, "Invalid mana value".to_owned()))
    );
    press(&mut app, &mut sheet, &[KeyCode::Backspace]);
    type_text(&mut app, &mut sheet, "3");
    press(&mut app, &mut sheet, &[KeyCode::Up]);
    assert_eq!(sheet.draft.cmc_equals(), Some(3.0));

    press(&mut app, &mut sheet, &[KeyCode::Char(' ')]);
    assert_eq!(
        sheet.draft.cmc_equals(),
        None,
        "Range drops the exact value"
    );
    press(&mut app, &mut sheet, &[KeyCode::Down]);
    type_text(&mut app, &mut sheet, "1");
    press(&mut app, &mut sheet, &[KeyCode::Down]);
    assert_eq!(sheet.draft.cmc_range(), None, "one end alone does nothing");
    type_text(&mut app, &mut sheet, "4");
    press(&mut app, &mut sheet, &[KeyCode::Enter]);
    assert_eq!(sheet.draft.cmc_range(), Some((1.0, 4.0)));
}

#[test]
fn pickers_search_their_source_pick_with_enter_and_drop_with_backspace() {
    let mut app = app();
    let mut flyer = card("Flyer");
    flyer.scryfall_data.keywords = Some(vec!["Flying".to_owned(), "Flash".to_owned()]);
    let config = SheetConfig {
        sources: Sources::deck(&[flyer]),
        ..plain()
    };
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), config);
    open_section(&mut app, &mut sheet, Section::OracleText);
    let picker = Target::Row(Row::Picker(PickerField::Include(MatchGroup::Keywords)));
    sheet.focus = sheet.targets().iter().position(|t| *t == picker).unwrap();
    type_text(&mut app, &mut sheet, "fl");
    press(&mut app, &mut sheet, &[KeyCode::Tab, KeyCode::Enter]);
    assert_eq!(
        sheet.draft.keywords_contains_any(),
        Some(&["flying".to_owned()][..])
    );
    press(&mut app, &mut sheet, &[KeyCode::Enter]);
    assert_eq!(
        sheet.draft.keywords_contains_any(),
        Some(&["flying".to_owned(), "flash".to_owned()][..]),
        "the picked one leaves the results"
    );
    press(
        &mut app,
        &mut sheet,
        &[KeyCode::Backspace, KeyCode::Backspace],
    );
    press(&mut app, &mut sheet, &[KeyCode::Backspace]);
    assert_eq!(
        sheet.draft.keywords_contains_any(),
        Some(&["flying".to_owned()][..])
    );
}

#[test]
fn steppers_and_tri_toggles_write_the_draft() {
    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::Combat);
    press(
        &mut app,
        &mut sheet,
        &[KeyCode::Down, KeyCode::Down, KeyCode::Char('+')],
    );
    assert_eq!(
        sheet.focused(),
        Some(Target::Row(Row::Stepper(StepRow::Exact(
            crate::domain::filter::fields::Stat::Power
        ))))
    );
    assert_eq!(sheet.draft.power_equals(), Some(1));
    open_section(&mut app, &mut sheet, Section::Config);
    press(&mut app, &mut sheet, &[KeyCode::Down, KeyCode::Char(' ')]);
    assert_eq!(
        Toggle::Playable.get(&sheet.draft),
        None,
        "tapping Yes again unsets"
    );
}

fn draw(app: &App, sheet: &FilterSheet, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|f| draw::draw(f, &app.shared, sheet, f.area()))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[test]
fn the_sheet_draws_its_sections_and_the_open_rows() {
    let mut app = app();
    app.shared.cards.catalog.claim(&[Kind::CardRoles]);
    let mut filter = CardQueryBuilder::default();
    filter.set_name_contains("bolt");
    let mut sheet = FilterSheet::new(&filter, plain());
    open_section(&mut app, &mut sheet, Section::Name);
    let screen = draw(&app, &sheet, 80, 40);
    for expected in [
        "Filter",
        "Name",
        "Oracle text",
        "Config",
        "Name contains",
        "bolt",
        "apply",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    assert!(
        screen.contains('×'),
        "the active section shows its clear mark"
    );
    draw(&app, &sheet, 10, 4);
}

/// Focuses the Oracle tags row `row` of the open section.
fn focus_row(sheet: &mut FilterSheet, row: impl Fn(Row) -> bool) {
    sheet.focus = sheet
        .targets()
        .iter()
        .position(|t| matches!(t, Target::Row(r) if row(*r)))
        .unwrap();
}

#[test]
fn the_dictionary_chips_open_the_dictionary_and_its_use_lands_in_the_list() {
    use crate::domain::filter::sections;

    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::OracleTags);
    assert!(draw(&app, &sheet, 80, 40).contains("Dictionary"));

    let include = PickerField::Include(MatchGroup::Tags);
    focus_row(&mut sheet, |r| {
        matches!(r, Row::Label { .. }) && sections::dictionary_for(r) == Some(include)
    });
    press(&mut app, &mut sheet, &[KeyCode::Char('d')]);
    assert_eq!(sheet.dictionary, Some(include));
    app.shared.caches.handoff.oracle_tag = Some("ramp".to_owned());
    let mut cx = Context::new(&mut app.shared);
    assert!(returned_from_dictionary(Some(&mut sheet), &mut cx));
    assert_eq!(
        sheet.draft.oracle_tags_contains_any(),
        Some(&["ramp".to_owned()][..])
    );
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Success, "Added to filter".to_owned()))
    );
    assert!(sheet.dictionary.is_none() && app.shared.caches.handoff.oracle_tag.is_none());

    sheet.dictionary = Some(include);
    app.shared.caches.handoff.oracle_tag = Some("ramp".to_owned());
    let mut cx = Context::new(&mut app.shared);
    sheet.take_dictionary_tag(&mut cx);
    assert_eq!(
        last_toast(&app),
        Some((ToastKind::Info, "Already in filter".to_owned()))
    );

    let exclude = PickerField::Exclude(MatchGroup::Tags);
    focus_row(&mut sheet, |r| r == Row::Picker(exclude));
    press(&mut app, &mut sheet, &[KeyCode::F(3)]);
    assert_eq!(sheet.dictionary, Some(exclude), "F3 in the search box");
    app.shared.caches.handoff.oracle_tag = Some("removal".to_owned());
    let mut cx = Context::new(&mut app.shared);
    sheet.take_dictionary_tag(&mut cx);
    assert_eq!(
        sheet.draft.oracle_tags_excludes(),
        Some(&["removal".to_owned()][..])
    );

    let mut cx = Context::new(&mut app.shared);
    assert!(
        !returned_from_dictionary(Some(&mut sheet), &mut cx),
        "a return from anything else leaves the sheet alone"
    );
}

#[test]
fn i_in_the_oracle_tags_section_explains_oracle_tags() {
    let mut app = app();
    let mut sheet = FilterSheet::new(&CardQueryBuilder::default(), plain());
    open_section(&mut app, &mut sheet, Section::OracleTags);
    assert!(draw(&app, &sheet, 80, 40).contains("Oracle tags ?"));
    press(&mut app, &mut sheet, &[KeyCode::Char('i')]);
    assert_eq!(
        app.shared.hint.as_ref().map(|h| h.content.clone()),
        Some(HintTopic::OracleTags.content())
    );
    app.shared.hint = None;
    open_section(&mut app, &mut sheet, Section::Name);
    press(&mut app, &mut sheet, &[KeyCode::Char('i')]);
    assert!(
        app.shared.hint.is_none(),
        "other sections have no explainer"
    );
}
