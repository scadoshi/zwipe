use std::time::{Duration, Instant};

use Color::{Blue, Red};
use crossterm::event::KeyCode;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::user::models::hints::HINT_SUGGESTED_COMMANDERS;
use zwipe_core::http::contracts::suggestions::{
    HttpColorAffinity, HttpCommanderSuggestions, HttpSharedTag, HttpSuggestionReasons,
};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::playstyle::{self, PlaystyleView};
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::Toast;
use crate::test_support::{card, create_opened_with, render, screen_text, user};

/// The app's copy for this hint, as it reads.
const SUGGESTED_HINT: &str = "• Swipe right starts a deck with that commander\n• Swipe left skips it for a week\n• Swipe up saves it to your commander maybeboard\n• Swipe down undoes the last swipe\nBest fit first, 100 deep. The eye shows why each one was suggested\nFilter narrows the stack to one color identity";
/// The app's copy for this hint, as it reads.
const COMMANDER_HINT: &str = "• Swipe right picks that commander\n• Swipe left skips it\n• Swipe up saves it to your commander maybeboard\n• Swipe down undoes the last swipe\nMost-played first. Filter narrows by color or anything else";

fn suggestion(name: &str, colors: &[Color]) -> HttpCommanderSuggestion {
    let mut card = card(name);
    card.scryfall_data.color_identity = colors.to_vec().into();
    HttpCommanderSuggestion {
        card,
        reasons: HttpSuggestionReasons {
            tags: vec![HttpSharedTag {
                slug: "goblin-typal".to_owned(),
                role: None,
            }],
            colors: vec![HttpColorAffinity {
                color: "R".to_owned(),
                decks: 2,
            }],
        },
    }
}

fn held() -> Vec<HttpCommanderSuggestion> {
    vec![
        suggestion("Krenko", &[Red]),
        suggestion("Talrand", &[Blue]),
        suggestion("Zada", &[Red]),
        suggestion("Kess", &[Blue]),
    ]
}

/// A signed-in user who has seen these screens' hints.
fn seen_user() -> zwipe_core::domain::user::User {
    let mut me = user(true);
    me.hints_shown
        .insert(HINT_SUGGESTED_COMMANDERS.to_owned(), true);
    me.hints_shown
        .insert(hint::keys::SWIPE_SELECT.to_owned(), true);
    me
}

/// Home, then Playstyle with [`held`] landed, then the swipe opened with `s`, nothing queued.
fn swiping() -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(seen_user());
    harness.open(playstyle::open);
    harness.apply(playstyle::Update::Loaded {
        ticket: 1,
        found: Box::new(HttpCommanderSuggestions {
            suggestions: held(),
            deck_count: 4,
            profile: None,
        }),
    });
    harness.press(KeyCode::Char('s'));
    harness.clear();
    harness
}

/// Home, then the swipe in Commander mode excluding `exclude`, nothing queued.
fn commander(exclude: HashSet<Uuid>) -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(seen_user());
    harness.open(|cx| open(cx, Deal::Commander { exclude }, DeckLimit::Open));
    harness.clear();
    harness
}

fn view(harness: &Harness) -> &CommanderSwipeView {
    match harness.screen() {
        Screen::CommanderSwipe(view) => view,
        other => panic!("not the commander swipe: {other:?}"),
    }
}

fn view_mut(harness: &mut Harness) -> &mut CommanderSwipeView {
    match harness.screen_mut() {
        Screen::CommanderSwipe(view) => view,
        _ => panic!("not the commander swipe"),
    }
}

fn playstyle_view(harness: &Harness) -> &PlaystyleView {
    match harness.screen() {
        Screen::Playstyle(view) => view,
        other => panic!("not Playstyle: {other:?}"),
    }
}

fn top(harness: &Harness) -> String {
    view(harness)
        .current()
        .map(|c| c.scryfall_data.name.clone())
        .unwrap_or_default()
}

fn oracle(harness: &Harness) -> Uuid {
    view(harness)
        .current()
        .unwrap()
        .scryfall_data
        .oracle_id
        .unwrap()
}

fn last_toast(harness: &Harness) -> Toast {
    harness.app.shared.toasts.items().last().unwrap().clone()
}

fn named(names: &[&str]) -> Vec<Card> {
    names.iter().map(|n| card(n)).collect()
}

#[test]
fn s_deals_the_held_list_under_the_show_rows_identity() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(playstyle::open);
    harness.app.shared.hint = None;
    harness.apply(playstyle::Update::Loaded {
        ticket: 1,
        found: Box::new(HttpCommanderSuggestions {
            suggestions: held(),
            deck_count: 4,
            profile: None,
        }),
    });
    harness.clear();
    match harness.screen_mut() {
        Screen::Playstyle(v) => v.identity = Some(vec![Red]),
        _ => panic!("not Playstyle"),
    }
    harness.press(KeyCode::Char('s'));
    let names: Vec<&str> = view(&harness)
        .cards
        .iter()
        .map(|c| c.scryfall_data.name.as_str())
        .collect();
    assert_eq!(names, ["Krenko", "Zada"]);
    assert_eq!(
        harness.pending(),
        [screens::Request::MarkHintShown(
            hint::keys::SWIPE_SELECT.to_owned()
        )]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, "Suggested commanders");
    assert_eq!(hint.content.text(), SUGGESTED_HINT);
    let text = screen_text(&harness.app);
    assert!(text.contains("Suggested commanders"));
}

#[test]
fn the_card_on_top_shows_its_text_and_why() {
    let harness = swiping();
    let text = screen_text(&harness.app);
    for expected in [
        "Krenko",
        "Why this commander?",
        "• Shares oracle tags with your decks",
        "Other tags:  goblin-typal ",
        "• You build in red (2 of your 4 decks)",
        "→ add  ← skip  ↑ maybeboard  ↓ undo  f Filter  r Refresh  ? Hint  Esc Back",
    ] {
        assert!(text.contains(expected), "missing {expected} in\n{text}");
    }
}

#[test]
fn right_selects_closes_and_starts_a_deck_and_the_host_refetches() {
    let mut harness = swiping();
    harness.press(KeyCode::Right);
    assert_eq!(create_opened_with(&harness), Some("Krenko"));
    let depth = harness.app.nav.depth();
    assert!(
        matches!(
            harness.app.nav.screens_mut().nth(depth.saturating_sub(2)),
            Some(Screen::Playstyle(_))
        ),
        "the swipe closed and Create opened over Playstyle"
    );
    assert!(
        harness
            .app
            .shared
            .toasts
            .items()
            .iter()
            .any(|t| t.kind == ToastKind::Success && t.message == SELECTED)
    );
    assert!(
        harness
            .pending()
            .contains(&playstyle::Request::Load { ticket: 2 }.into()),
        "closing the swipe refetches the list"
    );
}

#[test]
fn up_saves_and_down_takes_the_save_back() {
    let mut harness = swiping();
    let krenko = oracle(&harness);
    harness.press(KeyCode::Char('k'));
    assert_eq!(harness.pending(), [Request::Save(krenko).into()]);
    assert_eq!(top(&harness), "Talrand");
    harness.clear();
    harness.apply(Update::Saved);
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Success, SAVED)
    );
    assert!(toast.expires_at <= Instant::now() + Duration::from_millis(1500));
    harness.press(KeyCode::Char('j'));
    assert_eq!(top(&harness), "Krenko");
    assert_eq!(harness.pending(), [Request::Unsave(krenko).into()]);
    harness.apply(Update::Unsaved);
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Info, UNSAVED)
    );
    harness.press(KeyCode::Char('u'));
    assert_eq!(
        last_toast(&harness).message,
        NOTHING_TO_UNDO,
        "back at the first card"
    );
    harness.apply(Update::Failed("Offline".to_owned()));
    assert_eq!(last_toast(&harness).kind, ToastKind::Error);
}

#[test]
fn a_card_without_an_oracle_id_cannot_be_saved_and_undoes_as_a_skip() {
    let mut harness = swiping();
    view_mut(&mut harness).cards[0].scryfall_data.oracle_id = None;
    harness.press(KeyCode::Up);
    assert_eq!(harness.pending(), []);
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Error, CANT_SAVE)
    );
    assert_eq!(view(&harness).history, [Swiped::Skip]);
    harness.press(KeyCode::Down);
    assert_eq!(harness.pending(), []);
    assert_eq!(last_toast(&harness).message, UNDID_SKIP);
}

#[test]
fn left_in_suggested_dismisses_quietly_and_undo_lifts_it() {
    let mut harness = swiping();
    let krenko = oracle(&harness);
    harness.press(KeyCode::Char('h'));
    assert_eq!(harness.pending(), [Request::Dismiss(krenko).into()]);
    assert_eq!(last_toast(&harness).message, SKIPPED);
    harness.clear();
    harness.press(KeyCode::Down);
    assert_eq!(harness.pending(), [Request::Undismiss(krenko).into()]);
    assert_eq!(
        (last_toast(&harness).kind, last_toast(&harness).message),
        (ToastKind::Info, UNDID_SKIP.to_owned())
    );
}

#[test]
fn the_filter_redeals_behind_the_swiped_cards_and_the_host_takes_it() {
    let mut harness = swiping();
    harness.press(KeyCode::Left);
    harness.clear();
    harness.press(KeyCode::Char('f'));
    let text = screen_text(&harness.app);
    assert!(text.contains(" Show   All   U   R "), "{text}");
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    let pile = view(&harness).suggested().unwrap();
    assert_eq!(pile.identity, Some(vec![Blue]));
    let names: Vec<&str> = view(&harness)
        .cards
        .iter()
        .map(|c| c.scryfall_data.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["Krenko", "Talrand", "Kess"],
        "Krenko was swiped, so blue deals behind it"
    );
    assert_eq!(top(&harness), "Talrand");
    harness.press(KeyCode::Enter);
    assert_eq!(
        view(&harness).suggested().unwrap().identity,
        None,
        "the picked chip again is All"
    );
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    assert_eq!(
        view(&harness).suggested().unwrap().identity,
        Some(vec![Red])
    );
    harness.press(KeyCode::Esc);
    assert_eq!(view(&harness).filter, None);
    assert!(screen_text(&harness.app).contains("f Filter •"));
    harness.press(KeyCode::Esc);
    let host = playstyle_view(&harness);
    assert_eq!(host.identity, Some(vec![Red]), "one identity across both");
    assert_eq!(
        harness.pending(),
        [playstyle::Request::Load { ticket: 2 }.into()]
    );
}

#[test]
fn a_filter_with_nothing_left_says_so() {
    let mut harness = swiping();
    if let Mode::Suggested(pile) = &mut view_mut(&mut harness).mode {
        pile.held.retain(|s| s.card.scryfall_data.name != "Talrand");
        pile.held.retain(|s| s.card.scryfall_data.name != "Kess");
        pile.identity = Some(vec![Blue]);
    }
    view_mut(&mut harness).cards.clear();
    assert!(
        screen_text(&harness.app).contains("No mono-blue commanders in your top 100"),
        "{}",
        screen_text(&harness.app)
    );
}

#[test]
fn refresh_deals_again_from_the_top() {
    let mut harness = swiping();
    harness.press(KeyCode::Left);
    harness.press(KeyCode::Char('r'));
    assert_eq!(top(&harness), "Krenko");
    assert_eq!(view(&harness).history, []);
    assert_eq!(last_toast(&harness).message, REFRESHED);
}

#[test]
fn question_mark_shows_each_modes_hint() {
    let mut harness = swiping();
    harness.press(KeyCode::Char('?'));
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(
        (hint.content.title.as_str(), hint.content.text().as_str()),
        (SUGGESTED_TITLE, SUGGESTED_HINT)
    );
    harness.press(KeyCode::Enter);
    let mut harness = commander(HashSet::new());
    harness.press(KeyCode::Char('?'));
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(
        (hint.content.title.as_str(), hint.content.text().as_str()),
        (COMMANDER_TITLE, COMMANDER_HINT)
    );
}

#[test]
fn commander_mode_pages_most_played_first_without_the_excluded() {
    let excluded = card("Atraxa");
    let exclude = HashSet::from([excluded.scryfall_data.oracle_id.unwrap()]);
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(seen_user());
    harness.open(|cx| open(cx, Deal::Commander { exclude }, DeckLimit::Open));
    assert_eq!(
        harness.pending(),
        [Request::Search {
            ticket: 0,
            offset: 0,
            filter: Box::default(),
        }
        .into()]
    );
    let text = screen_text(&harness.app);
    assert!(text.contains("Swipe select"));
    assert!(text.contains('░'), "a ghost while the first page loads");
    harness.clear();
    let mut page = named(&["Edgar", "Krenko", "Ur-Dragon"]);
    page.insert(1, excluded);
    harness.apply(Update::Page {
        ticket: 0,
        offset: 0,
        cards: page,
    });
    let names: Vec<&str> = view(&harness)
        .cards
        .iter()
        .map(|c| c.scryfall_data.name.as_str())
        .collect();
    assert_eq!(names, ["Edgar", "Krenko", "Ur-Dragon"]);
    harness.press(KeyCode::Left);
    assert_eq!(
        harness.pending(),
        [Request::Search {
            ticket: 0,
            offset: PAGE,
            filter: Box::default(),
        }
        .into()],
        "two left is under the prefetch mark; a commander skip sends nothing"
    );
    harness.clear();
    harness.apply(Update::Page {
        ticket: 0,
        offset: PAGE,
        cards: Vec::new(),
    });
    let Mode::Commander(search) = &view(&harness).mode else {
        panic!("not commander mode");
    };
    assert!(search.exhausted);
    assert!(!search.loading);
    harness.press(KeyCode::Left);
    assert_eq!(harness.pending(), [], "nothing more to ask for");
}

#[test]
fn an_excluded_page_pulls_ahead_and_a_stale_page_is_dropped() {
    let excluded = card("Atraxa");
    let mut harness = commander(HashSet::from([excluded.scryfall_data.oracle_id.unwrap()]));
    harness.apply(Update::Page {
        ticket: 0,
        offset: 0,
        cards: vec![excluded],
    });
    assert_eq!(
        harness.pending(),
        [Request::Search {
            ticket: 0,
            offset: PAGE,
            filter: Box::default(),
        }
        .into()]
    );
    view_mut(&mut harness).refresh_for_test();
    harness.apply(Update::Page {
        ticket: 0,
        offset: PAGE,
        cards: named(&["Stale"]),
    });
    assert_eq!(view(&harness).cards, [], "a page from the old search");
    harness.apply(Update::SearchFailed { ticket: 1 });
    let Mode::Commander(search) = &view(&harness).mode else {
        panic!("not commander mode");
    };
    assert!(!search.loading);
    assert!(
        commander_query(&CardQueryBuilder::default(), PAGE)
            .build()
            .is_ok()
    );
}

impl CommanderSwipeView {
    /// Starts a new search the way Refresh does, outside a key press.
    fn refresh_for_test(&mut self) {
        if let Mode::Commander(search) = &mut self.mode {
            search.ticket = search.ticket.wrapping_add(1);
            search.offset = 0;
            search.loading = true;
        }
    }
}

#[test]
fn closing_commander_mode_leaves_no_identity() {
    let mut harness = commander(HashSet::new());
    harness.press(KeyCode::Esc);
    assert_eq!(
        harness.app.shared.caches.handoff.commander_swipe,
        Some(Closed::default())
    );
}

#[test]
fn redeal_keeps_the_swiped_in_front_then_the_new_pile_without_them() {
    fn key(name: &&'static str) -> &'static str {
        name
    }
    let dealt = ["Krenko", "Narset", "Zada", "Ghired"];
    let fresh = ["Zada", "Krenko", "Talrand"];
    assert_eq!(
        redeal(&dealt, 2, &fresh, key),
        vec!["Krenko", "Narset", "Zada", "Talrand"]
    );
    assert_eq!(redeal(&dealt, 0, &fresh, key), fresh.to_vec());
    assert_eq!(redeal(&dealt, 1, &[], key), vec!["Krenko"]);
    assert_eq!(redeal(&dealt, 9, &["Talrand"], key).len(), 5);
}

#[test]
fn tiny_terminals_do_not_panic() {
    let mut harness = swiping();
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
    harness.press(KeyCode::Char('f'));
    for (w, h) in [(8, 3), (1, 1)] {
        render(&harness.app, w, h);
    }
    let harness = commander(HashSet::new());
    for (w, h) in [(8, 3), (1, 1)] {
        render(&harness.app, w, h);
    }
}

#[test]
fn commander_modes_filter_is_the_card_filter_and_apply_deals_again() {
    use crate::domain::filter::sections::Section as FilterSection;

    let mut harness = commander(HashSet::new());
    harness.apply(Update::Page {
        ticket: 0,
        offset: 0,
        cards: named(&["Krenko", "Zada"]),
    });
    harness.press(KeyCode::Left);
    harness.press(KeyCode::Char('f'));
    harness.app.shared.hint = None;
    let sheet = view(&harness)
        .sheet
        .as_ref()
        .expect("f opens the card filter");
    assert!(!sheet.sections().contains(&FilterSection::Format));
    assert!(screen_text(&harness.app).contains(" Filter "));
    if let Some(sheet) = &mut view_mut(&mut harness).sheet {
        sheet.draft.set_name_contains("elf");
    }
    harness.clear();
    harness.press(KeyCode::Char('a'));
    assert!(view(&harness).sheet.is_none(), "Apply closes the sheet");
    let mut narrowed = CardQueryBuilder::default();
    narrowed.set_name_contains("elf");
    assert_eq!(
        harness.pending(),
        [Request::Search {
            ticket: 1,
            offset: 0,
            filter: Box::new(narrowed.clone()),
        }
        .into()],
        "Apply deals from the top with the filter"
    );
    assert_eq!(view(&harness).index, 0);
    assert_eq!(view(&harness).cards, []);
    assert_eq!(view(&harness).history, []);
    assert!(screen_text(&harness.app).contains("Filter •"));

    let query = commander_query(&narrowed, PAGE);
    assert_eq!(query.name_contains(), Some("elf"));
    assert_eq!(query.is_commander_in_format(), Some(&Format::Commander));
    assert_eq!(query.offset(), PAGE);
}

#[test]
fn skips_and_saves_fly_off_and_an_undo_flies_back_from_the_same_side() {
    use crate::domain::stack::Direction;
    use crate::inbound::widgets::card_motion::{CardMotion, Phase};

    let mut harness = swiping();
    let first = top(&harness);
    harness.press(KeyCode::Left);
    let motion = view(&harness).motion.clone().expect("a skip flies off");
    assert_eq!(motion.direction(), Direction::Left);
    assert!(matches!(motion.phase(), Phase::Exit(c) if c.scryfall_data.name == first));
    assert!(
        harness.app.nav.top().animating(),
        "the loop redraws while it flies"
    );
    render(&harness.app, 80, 30);

    let second = top(&harness);
    harness.press(KeyCode::Up);
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(motion.direction(), Direction::Up);
    assert!(matches!(motion.phase(), Phase::Exit(c) if c.scryfall_data.name == second));

    harness.press(KeyCode::Down);
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(
        (motion.direction(), motion.phase()),
        (Direction::Up, &Phase::Enter)
    );
    assert_eq!(top(&harness), second);
    assert_eq!(
        view(&harness).under().map(|c| c.scryfall_data.name.clone()),
        Some(top_after(&harness)),
        "the card it covers rests under it"
    );
    render(&harness.app, 80, 30);
    harness.press(KeyCode::Down);
    assert_eq!(
        view(&harness).motion.as_ref().map(CardMotion::direction),
        Some(Direction::Left)
    );

    harness.press(KeyCode::Right);
    assert!(
        !matches!(harness.screen(), Screen::CommanderSwipe(_)),
        "a pick closes the swipe"
    );
}

/// The name of the card under the top one.
fn top_after(harness: &Harness) -> String {
    let at = view(harness).index.saturating_add(1);
    view(harness).cards[at].scryfall_data.name.clone()
}

#[test]
fn the_swipe_keys_read_add_skip_maybeboard_undo_in_the_apps_swipe_colors() {
    use crate::test_support::word_colors;

    for mut harness in [swiping(), commander(HashSet::new())] {
        harness.apply(Update::Page {
            ticket: 0,
            offset: 0,
            cards: named(&["Krenko", "Zada"]),
        });
        let palette = harness.app.shared.palette;
        let row = "→ add  ← skip  ↑ maybeboard  ↓ undo";
        let text = screen_text(&harness.app);
        assert!(text.contains(row), "{text}");
        assert!(!text.contains("keep") && !text.contains("pass") && !text.contains("Pick"));
        assert_eq!(
            word_colors(&harness.app, row, &["add", "skip", "maybeboard", "undo"]),
            [
                palette.add.fg,
                palette.error.fg,
                palette.warning.fg,
                palette.tertiary.fg
            ]
        );
    }
}
