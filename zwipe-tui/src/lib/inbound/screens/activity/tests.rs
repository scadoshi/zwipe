use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use crossterm::event::KeyCode;
use reqwest::Url;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::user::models::hints::HINT_OTAG_EXAMPLES;
use zwipe_core::http::contracts::suggestions::{
    HttpActivityDay, HttpHardPass, HttpRoleRates, HttpSwipeOutcomes, HttpTagRates,
};

use super::*;
use crate::domain::activity::{role_rate_bars, tag_rate_bars, year_tags};
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home::cache::CachedMyCounts;
use crate::inbound::screens::printings::Mode;
use crate::outbound::account::SessionFile;
use crate::test_support::{card, render, screen_text, user};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn outcomes(added: u64, skipped: u64, maybed: u64, removed: u64) -> HttpSwipeOutcomes {
    HttpSwipeOutcomes {
        added,
        skipped,
        maybed,
        removed,
    }
}

fn tag(slug: &str, mine: HttpSwipeOutcomes, everyone: HttpSwipeOutcomes) -> HttpTagRates {
    HttpTagRates {
        tag: slug.to_owned(),
        role: "ramp".to_owned(),
        outcomes: mine,
        everyone,
    }
}

/// A year with three busy days, every outcome, two roles, two tags and one pass.
fn sample() -> HttpActivity {
    let through = date(2026, 10, 27);
    HttpActivity {
        through,
        days: vec![
            HttpActivityDay {
                day: date(2026, 10, 20),
                swipes: 12,
                searches: 1,
            },
            HttpActivityDay {
                day: date(2026, 10, 26),
                swipes: 3,
                searches: 0,
            },
            HttpActivityDay {
                day: through,
                swipes: 1_200,
                searches: 2,
            },
        ],
        outcomes: outcomes(5, 20, 1, 2),
        roles: vec![
            HttpRoleRates {
                role: "ramp".to_owned(),
                outcomes: outcomes(1, 2, 0, 0),
            },
            HttpRoleRates {
                role: "removal".to_owned(),
                outcomes: outcomes(9, 1, 0, 0),
            },
        ],
        tags: vec![
            tag("mana-rock", outcomes(1, 9, 0, 0), outcomes(80, 20, 0, 0)),
            tag("generic", outcomes(1, 9, 0, 0), outcomes(10, 90, 0, 0)),
        ],
        passes: vec![HttpHardPass {
            card: card("Sol Ring"),
            skipped: 3,
            commanders: vec!["Atraxa, Praetors' Voice".to_owned()],
        }],
    }
}

fn empty() -> HttpActivity {
    HttpActivity {
        through: date(2026, 10, 27),
        days: Vec::new(),
        outcomes: HttpSwipeOutcomes::default(),
        roles: Vec::new(),
        tags: Vec::new(),
        passes: Vec::new(),
    }
}

/// Home with Activity open over it, the hint already seen, and nothing queued.
fn on_activity() -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut me = user(true);
    me.hints_shown.insert(HINT_ACTIVITY.to_owned(), true);
    me.hints_shown.insert(HINT_OTAG_EXAMPLES.to_owned(), true);
    harness.app.shared.user = Some(me);
    harness.open(open);
    harness.clear();
    harness
}

/// [`on_activity`] with [`sample`] landed.
fn loaded() -> Harness {
    let mut harness = on_activity();
    harness.apply(Update::Loaded(Box::new(sample())));
    view_mut(&mut harness).heat_opened = Instant::now().checked_sub(Duration::from_secs(5));
    harness
}

fn view(harness: &Harness) -> &ActivityView {
    match harness.screen() {
        Screen::Activity(view) => view,
        other => panic!("not Activity: {other:?}"),
    }
}

fn view_mut(harness: &mut Harness) -> &mut ActivityView {
    match harness.screen_mut() {
        Screen::Activity(view) => view,
        _ => panic!("not Activity"),
    }
}

fn press(harness: &mut Harness, codes: &[KeyCode]) {
    for &code in codes {
        harness.press(code);
    }
}

#[test]
fn v_on_home_opens_activity_and_asks_for_everything_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.press(KeyCode::Char('v'));
    assert!(matches!(harness.screen(), Screen::Activity(_)));
    assert_eq!(
        harness.pending(),
        [
            Request::Load.into(),
            home::Request::MyMetrics.into(),
            oracle_tags::Request::Catalog.into(),
            screens::Request::MarkHintShown(HINT_ACTIVITY.to_owned()),
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.title, "Activity");
    assert_eq!(
        hint.body,
        "How you pick, read from your swipes\n• Activity maps your swipes and searches over the last year, and what the cards dealt to your decks came to\n• Card roles and Oracle tags show how often you add, skip, save or remove each one\n• Most skipped lists cards you skip that other players usually keep\nEach section's ? explains it"
    );
}

#[test]
fn fresh_counters_are_not_fetched_again() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let me = user(true);
    harness.app.shared.caches.my_counts = Some(CachedMyCounts {
        user_id: me.id,
        figures: Figures::default(),
        fetched_at: Instant::now(),
    });
    harness.app.shared.user = Some(me);
    harness.open(open);
    assert!(!harness.pending().contains(&home::Request::MyMetrics.into()));
}

#[test]
fn the_counters_roll_until_the_payload_lands() {
    let mut harness = on_activity();
    let me = harness.app.shared.user.as_ref().unwrap().id;
    harness.apply(home::Update::MyMetrics {
        user_id: me,
        figures: Figures {
            decks_created: 3,
            searches: 4,
            cards_swiped: 5,
        },
    });
    assert_eq!(view(&harness).figures(&harness.app.shared), None);
    assert!(!screen_text(&harness.app).contains("Live"));
    harness.apply(Update::Failed("down".to_owned()));
    assert!(view(&harness).figures(&harness.app.shared).is_some());
    assert!(view(&harness).figures_arrived.get().is_some());
    assert!(screen_text(&harness.app).contains(" Live   Only me "));
}

#[test]
fn a_failure_toasts_and_says_so() {
    let mut harness = on_activity();
    harness.apply(Update::Failed("The server is down".to_owned()));
    assert!(matches!(view(&harness).load, Load::Failed));
    let text = screen_text(&harness.app);
    assert!(text.contains(LOAD_FAILED));
    assert!(text.contains("The server is down"));
}

#[tokio::test]
async fn a_rejected_session_ends_it() {
    let client = ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap());
    let file = SessionFile::at(std::env::temp_dir().join("zwipe-tui-tests-no-session.json"));
    let account = Arc::new(Account::new(client, file));
    let update = perform(&account, Request::Load).await;
    assert!(matches!(update, Some(screens::Update::SessionEnded)));
}

#[test]
fn nothing_swiped_says_so() {
    let mut harness = on_activity();
    harness.apply(Update::Loaded(Box::new(empty())));
    assert!(screen_text(&harness.app).contains(NOTHING_YET));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Activity(_)));
}

#[test]
fn the_skeleton_ghosts_everything() {
    let harness = on_activity();
    let text = screen_text(&harness.app);
    assert!(text.contains('░'));
    for real in ["Mon", "Totals", "Card roles", "Most skipped", "Show"] {
        assert!(!text.contains(real), "{real} shows while loading");
    }
    assert!(view(&harness).animating());
}

#[test]
fn the_loaded_screen_draws_every_section_with_activity_open() {
    let harness = loaded();
    let v = view(&harness);
    assert_eq!(
        v.sections,
        [Section::Activity, Section::Roles, Section::Tags]
    );
    assert!(v.is_open(Section::Activity));
    assert_eq!(v.focus, Focus::Header(Section::Activity));
    let text = screen_text(&harness.app);
    for expected in [
        "> ▾ Activity  ?",
        "Mon",
        "Wed",
        "Fri",
        "Oct",
        " 1,215 swipes   3 searches   Last year ",
        " Totals ",
        "     20     ",
        "ADDED",
        "SKIPPED",
        "MAYBEBOARD",
        "REMOVED",
        "▸ Card roles",
        "▸ Oracle tags",
        "Most skipped  ?",
        "3 skips Sol Ring",
    ] {
        assert!(text.contains(expected), "missing {expected} in\n{text}");
    }
    assert!(
        v.loaded().unwrap().heat.cells.last().unwrap().peak,
        "the big day is a peak"
    );
}

#[test]
fn the_accordion_keeps_one_open_and_replays_the_sweep() {
    let mut harness = loaded();
    let first = view(&harness).heat_opened;
    press(&mut harness, &[KeyCode::Enter]);
    assert!(!view(&harness).is_open(Section::Activity));
    press(&mut harness, &[KeyCode::Char('j'), KeyCode::Enter]);
    let v = view(&harness);
    assert_eq!(v.focus, Focus::Header(Section::Roles));
    assert!(v.is_open(Section::Roles));
    assert!(!v.is_open(Section::Activity));
    view_mut(&mut harness).focus = Focus::Header(Section::Activity);
    std::thread::sleep(Duration::from_millis(2));
    press(&mut harness, &[KeyCode::Enter]);
    let v = view(&harness);
    assert!(v.is_open(Section::Activity) && !v.is_open(Section::Roles));
    assert!(v.heat_opened > first, "the sweep plays again");
}

#[test]
fn the_day_cursor_shows_a_day_then_the_year_again() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).focus, Focus::Heat);
    harness.press(KeyCode::Char('h'));
    assert_eq!(
        view(&harness).readout(),
        ["12 swipes", "1 search", "Oct 20"]
    );
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('k'));
    assert_eq!(
        view(&harness).readout(),
        ["3 swipes", "0 searches", "Oct 26"]
    );
    view_mut(&mut harness).picked_at = Instant::now().checked_sub(HEAT_PICK);
    assert_eq!(
        view(&harness).readout(),
        year_tags(&sample().days),
        "the year's totals come back after four seconds"
    );
    press(&mut harness, &[KeyCode::Char('k'), KeyCode::Char('k')]);
    assert_eq!(
        view(&harness).focus,
        Focus::Header(Section::Activity),
        "up past Sunday leaves the heatmap"
    );
}

#[test]
fn the_show_chips_switch_both_charts() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Header(Section::Roles);
    press(
        &mut harness,
        &[KeyCode::Enter, KeyCode::Char('j'), KeyCode::Char('l')],
    );
    let activity = sample();
    let v = view(&harness);
    assert_eq!(v.focus, Focus::Chips(Section::Roles));
    assert_eq!(v.rate, RateKind::Skipped);
    assert_eq!(
        v.bars(Section::Roles),
        role_rate_bars(RateKind::Skipped, &activity.roles, &activity.tags)
    );
    assert_eq!(
        v.bars(Section::Tags),
        tag_rate_bars(RateKind::Skipped, &activity.tags)
    );
    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(&mut harness, &[KeyCode::Enter]);
    let text = screen_text(&harness.app);
    assert!(text.contains("Show  Added   Skipped "), "{text}");
    assert!(text.contains(" mana-rock "), "{text}");
    view_mut(&mut harness).focus = Focus::Chips(Section::Tags);
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Char('l')]);
    assert_eq!(view(&harness).rate, RateKind::Removed, "the end stays put");
    assert!(
        screen_text(&harness.app).contains(activity::NONE_REMOVED),
        "no tag has a removal, so the tag chart says so"
    );
    press(&mut harness, &[KeyCode::Char('h')]);
    assert_eq!(view(&harness).rate, RateKind::Maybed);
}

#[test]
fn a_role_reveals_its_tags_and_a_tag_its_definition() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Header(Section::Roles);
    press(
        &mut harness,
        &[KeyCode::Enter, KeyCode::Char('j'), KeyCode::Char('j')],
    );
    assert_eq!(view(&harness).focus, Focus::Bar(Section::Roles, 0));
    press(&mut harness, &[KeyCode::Enter]);
    assert_eq!(view(&harness).open_role, None, "removal has no tags");
    press(&mut harness, &[KeyCode::Char('j'), KeyCode::Enter]);
    let v = view(&harness);
    assert_eq!(v.open_role.as_deref(), Some("ramp"));
    assert_eq!(v.role_tags.tags(), ["mana-rock", "generic"]);
    let text = screen_text(&harness.app);
    assert!(text.contains(" RAMP "), "{text}");
    assert!(text.contains(" mana-rock   generic "), "{text}");
    press(&mut harness, &[KeyCode::Tab]);
    assert!(screen_text(&harness.app).contains("No description yet"));
    press(&mut harness, &[KeyCode::Char('x')]);
    let Screen::OtagExamples(examples) = harness.screen() else {
        panic!("no examples");
    };
    assert_eq!(examples.slug, "mana-rock");
    harness.press(KeyCode::Esc);

    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(
        &mut harness,
        &[
            KeyCode::Enter,
            KeyCode::Char('j'),
            KeyCode::Char('l'),
            KeyCode::Char('j'),
            KeyCode::Enter,
        ],
    );
    assert_eq!(view(&harness).open_tag.as_deref(), Some("mana-rock"));
    assert!(screen_text(&harness.app).contains("x Examples"));
    press(&mut harness, &[KeyCode::Char('x')]);
    assert!(matches!(harness.screen(), Screen::OtagExamples(_)));
}

#[test]
fn question_mark_shows_the_open_sections_topic_and_h_the_hint() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('?'));
    assert_eq!(view(&harness).overlay, Some(Section::Activity.topic()));
    assert!(screen_text(&harness.app).contains("A lit square is one of your biggest days"));
    harness.press(KeyCode::Esc);
    assert_eq!(view(&harness).overlay, None);
    assert!(
        matches!(harness.screen(), Screen::Activity(_)),
        "Esc closed the topic, not the screen"
    );
    view_mut(&mut harness).focus = Focus::Header(Section::Roles);
    harness.press(KeyCode::Char('?'));
    assert_eq!(view(&harness).overlay, None, "a closed section has no ?");
    view_mut(&mut harness).focus = Focus::Passes;
    harness.press(KeyCode::Char('?'));
    assert_eq!(view(&harness).overlay, Some(passes_topic()));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('H'));
    assert_eq!(
        harness.app.shared.hint.as_ref().map(|h| h.body.as_str()),
        Some(HINT)
    );
}

#[test]
fn a_pass_row_expands_and_takes_a_picked_printing() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(&mut harness, &[KeyCode::Char('j')]);
    assert_eq!(view(&harness).focus, Focus::Passes);
    press(&mut harness, &[KeyCode::Enter]);
    let text = screen_text(&harness.app);
    assert!(text.contains("Skipped under"), "{text}");
    assert!(text.contains(" Atraxa, Praetors' Voice "), "{text}");
    let oracle_id = view(&harness).loaded().unwrap().activity.passes[0]
        .card
        .scryfall_data
        .oracle_id;
    press(&mut harness, &[KeyCode::Char('p')]);
    let Screen::Printings(printings) = harness.screen() else {
        panic!("no printings");
    };
    assert_eq!(printings.mode, Mode::Pick);
    let mut printing = card("Sol Ring");
    printing.scryfall_data.oracle_id = oracle_id;
    harness.app.shared.caches.handoff.printing = Some(printing.clone());
    harness.press(KeyCode::Esc);
    let v = view(&harness);
    let pass = &v.loaded().unwrap().activity.passes[0];
    assert_eq!(pass.card, printing);
    assert_eq!(v.rows.expanded(), Some(printing.scryfall_data.id));
    press(&mut harness, &[KeyCode::Char('k')]);
    assert_eq!(view(&harness).focus, Focus::Header(Section::Tags));
}

#[test]
fn esc_goes_back_home() {
    let mut harness = loaded();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn a_narrow_heatmap_crops_to_the_newest_weeks_and_follows_the_cursor() {
    let mut harness = loaded();
    let wide = render(&harness.app, 130, 60);
    let wide_text: String = wide
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(
        wide_text.contains("▆ ▆"),
        "two-cell squares when the year fits"
    );
    let narrow: String = render(&harness.app, 40, 60)
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(narrow.contains("Oct"));
    assert!(!narrow.contains("Jan"), "older months are cropped");
    harness.press(KeyCode::Char('j'));
    for _ in 0..60 {
        harness.press(KeyCode::Char('h'));
    }
    let cursor = view(&harness).cursor;
    assert_eq!(
        view(&harness).loaded().unwrap().heat.cells[cursor].column,
        1,
        "the year's first Tuesday, as it starts on a Wednesday"
    );
    let left: String = render(&harness.app, 40, 60)
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(left.contains("Nov"), "the crop follows the cursor left");
}

#[test]
fn tiny_terminals_do_not_panic() {
    let mut harness = on_activity();
    for (w, h) in [(8, 3), (1, 1)] {
        render(&harness.app, w, h);
    }
    harness.apply(Update::Loaded(Box::new(sample())));
    harness.press(KeyCode::Char('?'));
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
    harness.press(KeyCode::Esc);
    view_mut(&mut harness).focus = Focus::Passes;
    harness.press(KeyCode::Enter);
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
}

#[test]
fn empty_sections_stay_with_a_line_saying_what_fills_them() {
    let mut harness = on_activity();
    let mut only_days = empty();
    only_days.days.push(HttpActivityDay {
        day: only_days.through,
        swipes: 4,
        searches: 0,
    });
    harness.apply(Update::Loaded(Box::new(only_days)));
    let v = view(&harness);
    assert_eq!(
        v.sections,
        [Section::Activity, Section::Roles, Section::Tags]
    );
    let text = screen_text(&harness.app);
    assert!(text.contains("▸ Card roles"), "{text}");
    assert!(text.contains("▸ Oracle tags"), "{text}");
    assert!(text.contains(activity::PASSES_EMPTY), "{text}");

    view_mut(&mut harness).focus = Focus::Header(Section::Roles);
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains(activity::ROLES_EMPTY));
    harness.press(KeyCode::Char('j'));
    assert_eq!(
        view(&harness).focus,
        Focus::Header(Section::Tags),
        "no Show chips without roles"
    );
    press(&mut harness, &[KeyCode::Enter, KeyCode::Char('j')]);
    assert!(screen_text(&harness.app).contains(activity::TAGS_EMPTY));
    assert_eq!(view(&harness).focus, Focus::Passes);
    press(&mut harness, &[KeyCode::Enter, KeyCode::Char('j')]);
    assert!(matches!(harness.screen(), Screen::Activity(_)));
    harness.press(KeyCode::Char('?'));
    assert_eq!(view(&harness).overlay, Some(passes_topic()));
}

#[test]
fn a_role_with_no_tags_is_a_grey_chip() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Header(Section::Roles);
    harness.press(KeyCode::Enter);
    let terminal = render(&harness.app, 100, 40);
    let buffer = terminal.backend().buffer();
    let symbols: Vec<&str> = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    let text = symbols.concat();
    let at = text.find(" REMOV ").unwrap();
    let cell = &buffer.content()[text[..at].chars().count() + 1];
    assert!(
        cell.modifier.contains(ratatui::style::Modifier::REVERSED),
        "a chip"
    );
    assert!(
        cell.modifier.contains(ratatui::style::Modifier::DIM),
        "grey"
    );
    assert_eq!(
        Section::Roles.topic().bullets[1],
        "Tap a role to see the tags you swiped under it. A grey role has none yet"
    );
}
