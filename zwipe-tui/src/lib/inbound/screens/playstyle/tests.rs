use std::sync::Arc;
use std::time::{Duration, Instant};

use Color::{Blue, Green, Red, White};
use crossterm::event::KeyCode;
use reqwest::Url;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::user::models::hints::HINT_OTAG_EXAMPLES;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpColorAffinity, HttpColorCards, HttpColorIdentityCount, HttpColorMap,
    HttpCommanderSuggestion, HttpCommanderSuggestions, HttpProfileRole, HttpProfileTag,
    HttpSharedTag, HttpStaple, HttpSuggestionReasons,
};

use super::*;
use crate::domain::playstyle;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::printings::Mode;
use crate::inbound::widgets::toast::Toast;
use crate::outbound::account::SessionFile;
use crate::test_support::{card, render, screen_text, user};

/// A card named `name` in `colors`.
fn colored(name: &str, colors: &[Color]) -> Card {
    let mut card = card(name);
    card.scryfall_data.color_identity = colors.to_vec().into();
    card
}

/// A suggestion with one shared tag under ramp and one color reason.
fn suggestion(name: &str, colors: &[Color]) -> HttpCommanderSuggestion {
    HttpCommanderSuggestion {
        card: colored(name, colors),
        reasons: HttpSuggestionReasons {
            tags: vec![HttpSharedTag {
                slug: "mana-rock".to_owned(),
                role: Some("ramp".to_owned()),
            }],
            colors: vec![HttpColorAffinity {
                color: "R".to_owned(),
                decks: 3,
            }],
        },
    }
}

/// Twelve red commanders, then a blue one and an Azorius one.
fn held() -> Vec<HttpCommanderSuggestion> {
    let mut held: Vec<HttpCommanderSuggestion> = (0..12)
        .map(|i| suggestion(&format!("Red {i:02}"), &[Red]))
        .collect();
    held.push(suggestion("Talrand", &[Blue]));
    held.push(suggestion("Brago", &[Blue, White]));
    held
}

fn staple(name: &str, type_line: &str, colors: &[Color], decks: u32) -> HttpStaple {
    let mut card = colored(name, colors);
    card.scryfall_data.type_line = Some(type_line.to_owned());
    HttpStaple { card, decks }
}

fn profile() -> HttpBuilderProfile {
    HttpBuilderProfile {
        deck_count: 4,
        colors: vec![
            HttpColorAffinity {
                color: "R".to_owned(),
                decks: 3,
            },
            HttpColorAffinity {
                color: "U".to_owned(),
                decks: 1,
            },
        ],
        top_identity: Some(HttpColorIdentityCount {
            colors: vec!["U".to_owned(), "W".to_owned()],
            decks: 2,
        }),
        tags: vec![
            HttpProfileTag {
                slug: "mana-rock".to_owned(),
                role: Some("ramp".to_owned()),
                decks: 4,
                cards: 12,
                score: 2.0,
            },
            HttpProfileTag {
                slug: "goblin-typal".to_owned(),
                role: None,
                decks: 2,
                cards: 30,
                score: 1.0,
            },
        ],
        roles: vec![
            HttpProfileRole {
                role: "ramp".to_owned(),
                decks: 4,
            },
            HttpProfileRole {
                role: "removal".to_owned(),
                decks: 2,
            },
        ],
        staples: vec![
            staple("Sol Ring", "Artifact", &[], 4),
            staple("Lightning Bolt", "Instant", &[Red], 3),
            staple("Goblin Guide", "Creature — Goblin Scout", &[Red], 2),
        ],
        color_map: Some(HttpColorMap {
            colors: vec![
                HttpColorCards {
                    color: "R".to_owned(),
                    cards: 120,
                },
                HttpColorCards {
                    color: "U".to_owned(),
                    cards: 40,
                },
            ],
            colorless_cards: 9,
            pairs: Vec::new(),
        }),
    }
}

fn found(suggestions: Vec<HttpCommanderSuggestion>) -> Box<HttpCommanderSuggestions> {
    Box::new(HttpCommanderSuggestions {
        suggestions,
        deck_count: 4,
        profile: Some(profile()),
    })
}

/// Home with Playstyle open over it, the hints already seen, and nothing queued.
fn on_playstyle(verified: bool) -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut me = user(verified);
    me.hints_shown
        .insert(HINT_SUGGESTED_COMMANDERS.to_owned(), true);
    me.hints_shown.insert(HINT_OTAG_EXAMPLES.to_owned(), true);
    harness.app.shared.user = Some(me);
    harness.open(open);
    harness.clear();
    harness
}

/// [`on_playstyle`] with [`held`] landed on the first ticket.
fn loaded() -> Harness {
    let mut harness = on_playstyle(true);
    harness.apply(Update::Loaded {
        ticket: 1,
        found: found(held()),
    });
    view_mut(&mut harness).map_opened = Instant::now().checked_sub(Duration::from_secs(5));
    harness
}

fn view(harness: &Harness) -> &PlaystyleView {
    match harness.screen() {
        Screen::Playstyle(view) => view,
        other => panic!("not Playstyle: {other:?}"),
    }
}

fn view_mut(harness: &mut Harness) -> &mut PlaystyleView {
    match harness.screen_mut() {
        Screen::Playstyle(view) => view,
        _ => panic!("not Playstyle"),
    }
}

fn press(harness: &mut Harness, codes: &[KeyCode]) {
    for &code in codes {
        harness.press(code);
    }
}

fn names(harness: &Harness) -> Vec<String> {
    view(harness)
        .shown()
        .iter()
        .map(|s| s.card.scryfall_data.name.clone())
        .collect()
}

fn toasts(harness: &Harness) -> Vec<(ToastKind, String)> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t: &Toast| (t.kind, t.message.clone()))
        .collect()
}

fn last_toast(harness: &Harness) -> Toast {
    harness.app.shared.toasts.items().last().unwrap().clone()
}

/// Puts the focus on the list and expands its first row.
fn expand_first(harness: &mut Harness) {
    view_mut(harness).focus = Focus::List;
    view_mut(harness).rows.set_focused(0);
    harness.press(KeyCode::Enter);
}

fn oracle_of(harness: &Harness, index: usize) -> Uuid {
    view(harness).shown()[index]
        .card
        .scryfall_data
        .oracle_id
        .unwrap()
}

#[test]
fn p_on_home_opens_playstyle_and_asks_for_everything_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.press(KeyCode::Char('p'));
    assert!(matches!(harness.screen(), Screen::Playstyle(_)));
    assert_eq!(
        harness.pending(),
        [
            Request::Load { ticket: 1 }.into(),
            Request::DeckProfiles.into(),
            oracle_tags::Request::Catalog.into(),
            screens::Request::MarkHintShown(HINT_SUGGESTED_COMMANDERS.to_owned()),
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.title, "Playstyle");
    assert_eq!(
        hint.body,
        "Your colors, card roles and top oracle tags, read from every deck you own in any format\n• Create deck begins a deck with that commander\n• Maybeboard saves it to your commander maybeboard\n• Remove hides it for a week and pulls the next one up\n• Swipe deals the same ranking: right starts a deck, up saves, left skips it\n• Show narrows the list to one color identity\nBelow them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking"
    );
}

#[test]
fn the_skeleton_ghosts_everything() {
    let harness = on_playstyle(true);
    let text = screen_text(&harness.app);
    assert!(text.contains('░'));
    for real in [
        "Color map",
        "Summary",
        "Show",
        "All",
        "Suggested commanders",
        "Staples",
    ] {
        assert!(!text.contains(real), "{real} shows while loading");
    }
    assert!(text.contains("M More"));
    assert!(view(&harness).animating());
}

#[test]
fn the_loaded_screen_draws_the_profile_list_and_staples() {
    let harness = loaded();
    let v = view(&harness);
    assert_eq!(
        v.sections,
        [
            Section::ColorMap,
            Section::Summary,
            Section::Roles,
            Section::Tags
        ]
    );
    assert!(v.is_open(Section::ColorMap), "Color map opens on entry");
    assert_eq!(v.focus, Focus::Header(Section::ColorMap));
    assert_eq!(names(&harness).len(), 10, "the top ten of the held list");
    let text = render_text(&harness, 110, 90);
    for expected in [
        "> ▾ Color map  ?",
        "██",
        "Colorless 9",
        "▸ Summary",
        "▸ Card roles",
        "▸ Oracle tags",
        " Show   All   U   R   Azorius WU ",
        "Suggested commanders  ?",
        "▸ 1 Red 00",
        "▸ 1 Red 09",
        " Type   All   Creature   Artifact   Instant ",
        " Color   All   R   Colorless ",
        "Staples  ?",
        "100% Sol Ring",
        "75% Lightning Bolt",
        "50% Goblin Guide",
    ] {
        assert!(text.contains(expected), "missing {expected} in\n{text}");
    }
    assert!(!text.contains("Red 10"), "the eleventh is held, not shown");
}

/// `app` drawn at `width` by `height`, every cell's symbol in order.
fn render_text(harness: &Harness, width: u16, height: u16) -> String {
    render(&harness.app, width, height)
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[test]
fn summary_shows_decks_and_the_top_colors() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Header(Section::Summary);
    harness.press(KeyCode::Enter);
    assert!(view(&harness).is_open(Section::Summary));
    assert!(!view(&harness).is_open(Section::ColorMap), "one at a time");
    let text = render_text(&harness, 110, 90);
    for expected in [
        "Decks",
        "Top colors",
        "Azorius WU (2)",
        "Top color",
        "Red R (3)",
    ] {
        assert!(text.contains(expected), "missing {expected} in\n{text}");
    }
}

#[test]
fn an_old_ticket_never_lands() {
    let mut harness = loaded();
    view_mut(&mut harness).ticket = 2;
    harness.apply(Update::Loaded {
        ticket: 1,
        found: found(vec![suggestion("Stale", &[Green])]),
    });
    assert_ne!(names(&harness), ["Stale"], "the older reply is dropped");
    harness.apply(Update::Loaded {
        ticket: 2,
        found: found(vec![suggestion("Fresh", &[Green])]),
    });
    assert_eq!(names(&harness), ["Fresh"]);
    assert!(!view(&harness).load_out);
}

#[test]
fn a_failed_first_load_says_so_and_a_failed_refetch_keeps_the_rows() {
    let mut harness = on_playstyle(true);
    harness.apply(Update::LoadFailed {
        ticket: 1,
        message: "The server is down".to_owned(),
    });
    assert!(view(&harness).load_failed);
    let text = screen_text(&harness.app);
    assert!(text.contains(LOAD_FAILED));
    assert!(text.contains("The server is down"));
    let toast = last_toast(&harness);
    assert_eq!(toast.kind, ToastKind::Error);
    assert!(
        toast.expires_at > Instant::now() + Duration::from_secs(4),
        "TOAST_NORMAL"
    );

    let mut harness = loaded();
    view_mut(&mut harness).ticket = 2;
    harness.apply(Update::LoadFailed {
        ticket: 2,
        message: "Offline".to_owned(),
    });
    assert!(!view(&harness).load_failed, "rows on screen stay");
    assert_eq!(names(&harness).len(), 10);
}

#[test]
fn no_decks_is_a_cold_start() {
    let mut harness = on_playstyle(true);
    harness.apply(Update::Loaded {
        ticket: 1,
        found: Box::new(HttpCommanderSuggestions {
            suggestions: Vec::new(),
            deck_count: 0,
            profile: None,
        }),
    });
    let text = screen_text(&harness.app);
    assert!(text.contains(COLD_START));
    assert!(!text.contains("j/k Move"));
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).stops(), []);
}

#[test]
fn nothing_suggested_says_so() {
    let mut harness = on_playstyle(true);
    harness.apply(Update::Loaded {
        ticket: 1,
        found: found(Vec::new()),
    });
    assert!(render_text(&harness, 110, 90).contains(NO_SUGGESTIONS));
}

#[test]
fn the_show_row_narrows_the_list_and_its_picked_chip_returns_to_all() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Show;
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(view(&harness).identity, Some(vec![Blue]));
    assert_eq!(names(&harness), ["Talrand"]);
    press(&mut harness, &[KeyCode::Enter]);
    assert_eq!(
        view(&harness).identity,
        None,
        "the picked chip again is All"
    );
    assert_eq!(view(&harness).show_cursor, 0);
    assert_eq!(names(&harness).len(), 10);
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    let talrand = oracle_of(&harness, 0);
    harness.apply(Update::Dismissed(talrand));
    assert_eq!(
        view(&harness).identities()[0],
        vec![Blue],
        "the chosen chip stays after its last commander goes"
    );
    assert!(render_text(&harness, 110, 90).contains(&playstyle::empty_identity_line(&[Blue])));
    assert_eq!(
        view(&harness).focus,
        Focus::Show,
        "the list's stop is gone, the focus stays on the chips"
    );
}

#[test]
fn maybeboard_and_remove_send_their_calls_and_drop_the_row() {
    let mut harness = loaded();
    expand_first(&mut harness);
    let first = oracle_of(&harness, 0);
    harness.clear();
    harness.press(KeyCode::Char('m'));
    assert_eq!(harness.pending(), [Request::Maybeboard(first).into()]);
    harness.clear();
    harness.apply(Update::Maybeboarded(first));
    assert!(!names(&harness).contains(&"Red 00".to_owned()));
    assert_eq!(names(&harness).len(), 10, "the next held one moves up");
    assert_eq!(
        harness.pending(),
        [],
        "thirteen left under All, so no refetch"
    );
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Success, ADDED)
    );

    expand_first(&mut harness);
    let second = oracle_of(&harness, 0);
    harness.press(KeyCode::Char('r'));
    assert_eq!(harness.pending(), [Request::Dismiss(second).into()]);
    harness.clear();
    harness.apply(Update::Dismissed(second));
    assert_eq!(
        toasts(&harness).last(),
        Some(&(ToastKind::Info, SKIPPED_FOR_A_WEEK.to_owned()))
    );
    assert_eq!(names(&harness).len(), 10);
    assert_eq!(view(&harness).entries.len(), 12);
}

#[test]
fn a_drop_refetches_when_the_list_runs_short_or_a_load_is_out() {
    let mut harness = loaded();
    view_mut(&mut harness).load_out = true;
    let first = oracle_of(&harness, 0);
    harness.apply(Update::Dismissed(first));
    assert_eq!(
        harness.pending(),
        [Request::Load { ticket: 2 }.into()],
        "a load from before the drop would bring the row back"
    );
    harness.clear();
    view_mut(&mut harness).load_out = false;
    view_mut(&mut harness).identity = Some(vec![Red]);
    let red = oracle_of(&harness, 0);
    harness.apply(Update::Maybeboarded(red));
    assert_eq!(harness.pending(), [], "ten red left is a full list");
    let red = oracle_of(&harness, 0);
    harness.apply(Update::Maybeboarded(red));
    assert_eq!(
        harness.pending(),
        [Request::Load { ticket: 3 }.into()],
        "nine red left is short"
    );
    harness.clear();
    harness.apply(Update::ActionFailed("Nope".to_owned()));
    assert_eq!(
        toasts(&harness).last(),
        Some(&(ToastKind::Error, "Nope".to_owned()))
    );
}

#[test]
fn create_deck_opens_with_the_card_or_warns_at_the_deck_limit() {
    let mut harness = loaded();
    expand_first(&mut harness);
    harness.press(KeyCode::Char('c'));
    assert_eq!(
        toasts(&harness).last(),
        Some(&(
            ToastKind::Info,
            "Create deck opens with Red 00 once deck creation lands".to_owned()
        ))
    );

    let mut harness = on_playstyle(false);
    harness.apply(Update::Loaded {
        ticket: 1,
        found: found(held()),
    });
    harness.apply(Update::DeckProfiles(1));
    expand_first(&mut harness);
    harness.press(KeyCode::Char('c'));
    let toast = last_toast(&harness);
    assert_eq!(
        (toast.kind, toast.message.as_str()),
        (ToastKind::Warning, VERIFY_TO_CREATE)
    );
    assert!(
        toast.expires_at > Instant::now() + Duration::from_secs(7),
        "TOAST_LONG"
    );
    assert_eq!(DeckLimit::of(Some(&user(false)), None), DeckLimit::Open);
    assert_eq!(DeckLimit::of(Some(&user(false)), Some(0)), DeckLimit::Open);
    assert_eq!(DeckLimit::of(Some(&user(true)), Some(3)), DeckLimit::Open);
}

#[test]
fn w_unfolds_why_this_commander() {
    let mut harness = loaded();
    expand_first(&mut harness);
    let folded = render_text(&harness, 110, 90);
    assert!(folded.contains("w  Why this commander? ▸"), "{folded}");
    assert!(!folded.contains(SHARES_TAGS));
    assert!(folded.contains("i Image  p Printing  c Create deck  m Maybeboard  r Remove"));
    harness.press(KeyCode::Char('w'));
    let open = render_text(&harness, 110, 90);
    for expected in [
        "Why this commander? ▾",
        "• Shares oracle tags with your decks",
        "Ramp:  mana-rock ",
        "• You build in red (3 of your 4 decks)",
    ] {
        assert!(open.contains(expected), "missing {expected} in\n{open}");
    }
    harness.press(KeyCode::Char('w'));
    assert_eq!(view(&harness).why_open, None);
}

#[test]
fn the_staples_type_and_color_rows_combine() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Types;
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(view(&harness).staple_type, Some(CardType::Creature));
    let staples: Vec<String> = view(&harness)
        .matching_staples()
        .iter()
        .map(|s| s.card.scryfall_data.name.clone())
        .collect();
    assert_eq!(staples, ["Goblin Guide"]);
    press(
        &mut harness,
        &[KeyCode::Char('j'), KeyCode::Char('l'), KeyCode::Char('l')],
    );
    press(&mut harness, &[KeyCode::Enter]);
    assert_eq!(view(&harness).staple_color, Some(StapleColor::Colorless));
    assert_eq!(view(&harness).matching_staples(), Vec::<HttpStaple>::new());
    assert!(render_text(&harness, 110, 90).contains(NO_STAPLES));
    press(&mut harness, &[KeyCode::Enter]);
    assert_eq!(view(&harness).staple_color, None, "again is All");
    assert_eq!(view(&harness).color_cursor, 0);
    press(&mut harness, &[KeyCode::Char('j'), KeyCode::Enter]);
    assert_eq!(view(&harness).focus, Focus::Staples);
    assert!(view(&harness).staple_rows.expanded().is_some());
    assert!(render_text(&harness, 110, 90).contains(" 2 of 4 decks "));
}

#[test]
fn clear_skips_asks_first_then_clears_and_refetches() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('M'));
    assert!(matches!(view(&harness).overlay, Some(Overlay::More(_))));
    let text = screen_text(&harness.app);
    assert!(text.contains("More actions"), "{text}");
    assert!(text.contains("c  Clear skips "), "{text}");
    harness.press(KeyCode::Char('c'));
    let text = screen_text(&harness.app);
    assert!(text.contains("can't be undone."), "{text}");
    assert!(text.contains(" Cancel (n) "));
    assert!(text.contains(" Clear (y) "));
    harness.press(KeyCode::Esc);
    assert!(
        matches!(view(&harness).overlay, Some(Overlay::More(_))),
        "Cancel goes back to the sheet"
    );
    assert_eq!(harness.pending(), []);
    press(&mut harness, &[KeyCode::Char('c'), KeyCode::Char('y')]);
    assert_eq!(view(&harness).overlay, None, "the sheet closes too");
    assert_eq!(harness.pending(), [Request::ClearSkips.into()]);
    harness.clear();
    harness.apply(Update::SkipsCleared);
    assert_eq!(
        toasts(&harness).last(),
        Some(&(ToastKind::Info, SKIPS_CLEARED.to_owned()))
    );
    assert_eq!(harness.pending(), [Request::Load { ticket: 2 }.into()]);
    harness.press(KeyCode::Char('M'));
    harness.press(KeyCode::Esc);
    assert_eq!(view(&harness).overlay, None);
    assert!(matches!(harness.screen(), Screen::Playstyle(_)));
}

#[test]
fn question_mark_shows_each_topic_and_h_the_hint() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('?'));
    assert_eq!(
        view(&harness).overlay,
        Some(Overlay::Info(home::keys::color_map_topic()))
    );
    harness.press(KeyCode::Esc);
    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(&mut harness, &[KeyCode::Enter, KeyCode::Char('?')]);
    assert_eq!(
        view(&harness).overlay,
        Some(Overlay::Info(Section::Tags.topic().unwrap()))
    );
    harness.press(KeyCode::Esc);
    press(
        &mut harness,
        &[
            KeyCode::Char('j'),
            KeyCode::Char('l'),
            KeyCode::Char('l'),
            KeyCode::Char('?'),
        ],
    );
    assert_eq!(view(&harness).focus, Focus::TagHead);
    assert_eq!(
        view(&harness).overlay,
        Some(Overlay::Info(Column::Impact.topic()))
    );
    assert!(screen_text(&harness.app).contains("Your top tag scores 100"));
    harness.press(KeyCode::Esc);
    view_mut(&mut harness).focus = Focus::List;
    harness.press(KeyCode::Char('?'));
    assert_eq!(view(&harness).overlay, Some(Overlay::Info(list_topic())));
    harness.press(KeyCode::Esc);
    view_mut(&mut harness).focus = Focus::Header(Section::Summary);
    harness.press(KeyCode::Char('?'));
    assert_eq!(view(&harness).overlay, None, "Summary has no ?");
    harness.press(KeyCode::Char('H'));
    assert_eq!(
        harness.app.shared.hint.as_ref().map(|h| h.body.as_str()),
        Some(HINT)
    );
}

#[test]
fn a_role_reveals_its_tags_and_a_table_tag_its_definition() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Header(Section::Roles);
    press(&mut harness, &[KeyCode::Enter, KeyCode::Char('j')]);
    assert_eq!(view(&harness).focus, Focus::Role(0));
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).open_role.as_deref(), Some("ramp"));
    let text = render_text(&harness, 110, 90);
    assert!(text.contains(" RAMP "), "{text}");
    assert!(text.contains(" REMOV "), "{text}");
    assert!(text.contains(" mana-rock "), "{text}");
    press(&mut harness, &[KeyCode::Tab, KeyCode::Char('x')]);
    let Screen::OtagExamples(examples) = harness.screen() else {
        panic!("no examples");
    };
    assert_eq!(examples.slug, "mana-rock");
    harness.press(KeyCode::Esc);

    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(
        &mut harness,
        &[KeyCode::Enter, KeyCode::Char('j'), KeyCode::Char('j')],
    );
    let table = render_text(&harness, 110, 90);
    assert!(table.contains("Tag"), "{table}");
    assert!(table.contains("Decks ?"), "{table}");
    assert!(table.contains("Cards ?"), "{table}");
    assert!(table.contains("Impact ?"), "{table}");
    assert!(table.contains(" 100 "), "{table}");
    assert!(table.contains(" 50 "), "{table}");
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).open_tag.as_deref(), Some("mana-rock"));
    assert!(render_text(&harness, 110, 90).contains("No description yet"));
    harness.press(KeyCode::Char('x'));
    assert!(matches!(harness.screen(), Screen::OtagExamples(_)));
}

#[test]
fn a_picked_printing_swaps_into_its_row() {
    let mut harness = loaded();
    expand_first(&mut harness);
    let oracle_id = oracle_of(&harness, 0);
    harness.press(KeyCode::Char('p'));
    let Screen::Printings(printings) = harness.screen() else {
        panic!("no printings");
    };
    assert_eq!(printings.mode, Mode::Pick);
    let mut printing = colored("Red 00", &[Red]);
    printing.scryfall_data.oracle_id = Some(oracle_id);
    harness.app.shared.caches.handoff.printing = Some(printing.clone());
    harness.press(KeyCode::Esc);
    let v = view(&harness);
    assert_eq!(v.entries[0].card, printing);
    assert_eq!(v.rows.expanded(), Some(printing.scryfall_data.id));
}

#[test]
fn moving_runs_through_every_stop_and_rows_hand_the_focus_on() {
    let mut harness = loaded();
    let stops = view(&harness).stops();
    assert_eq!(
        stops,
        [
            Focus::Header(Section::ColorMap),
            Focus::Header(Section::Summary),
            Focus::Header(Section::Roles),
            Focus::Header(Section::Tags),
            Focus::Show,
            Focus::List,
            Focus::Types,
            Focus::Colors,
            Focus::Staples,
        ]
    );
    view_mut(&mut harness).focus = Focus::Show;
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).focus, Focus::List);
    for _ in 0..9 {
        harness.press(KeyCode::Char('j'));
    }
    assert_eq!(view(&harness).rows.focused(), 9);
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).focus, Focus::Types, "down from the last row");
    harness.press(KeyCode::Char('k'));
    assert_eq!(view(&harness).focus, Focus::List);
    assert_eq!(view(&harness).rows.focused(), 9, "back in at the last row");
}

#[test]
fn esc_goes_back_home() {
    let mut harness = loaded();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[tokio::test]
async fn a_rejected_session_ends_it() {
    let client = ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap());
    let file = SessionFile::at(std::env::temp_dir().join("zwipe-tui-tests-no-session.json"));
    let account = Arc::new(Account::new(client, file));
    for request in [
        Request::Load { ticket: 1 },
        Request::DeckProfiles,
        Request::ClearSkips,
    ] {
        let update = perform(&account, request).await;
        assert!(matches!(update, Some(screens::Update::SessionEnded)));
    }
}

#[test]
fn a_long_chip_row_wraps_between_chips() {
    let palette = crate::domain::theme::Palette::terminal();
    let identities = vec![
        vec![Red],
        vec![Blue, White],
        vec![Blue, Red],
        vec![White, Blue, Red],
    ];
    let line = draw::show_line(&palette, &identities, None, 0, false);
    let lines = draw::wrap_at_gaps(line.clone(), 30);
    assert!(lines.len() > 1);
    let texts: Vec<String> = lines.iter().map(ToString::to_string).collect();
    assert!(texts.iter().all(|t| t.chars().count() <= 30), "{texts:?}");
    assert_eq!(texts.concat(), line.to_string(), "nothing lost or split");
    assert_eq!(draw::wrap_at_gaps(line, 500).len(), 1);
}

#[test]
fn tiny_terminals_do_not_panic() {
    let mut harness = on_playstyle(true);
    for (w, h) in [(8, 3), (1, 1)] {
        render(&harness.app, w, h);
    }
    harness.apply(Update::Loaded {
        ticket: 1,
        found: found(held()),
    });
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
    expand_first(&mut harness);
    harness.press(KeyCode::Char('w'));
    harness.press(KeyCode::Char('M'));
    harness.press(KeyCode::Char('c'));
    for (w, h) in [(8, 3), (1, 1), (20, 6)] {
        render(&harness.app, w, h);
    }
}
