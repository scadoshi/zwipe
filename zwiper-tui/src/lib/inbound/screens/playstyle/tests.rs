use std::sync::Arc;
use std::time::{Duration, Instant};

use Color::{Blue, Green, Red, White};
use crossterm::event::KeyCode;
use reqwest::Url;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::user::models::hints::HINT_OTAG_EXAMPLES;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpColorAffinity, HttpColorCards, HttpColorIdentityCount, HttpColorMap,
    HttpCommanderSuggestion, HttpCommanderSuggestions, HttpProfileRole, HttpProfileTag,
    HttpSharedTag, HttpStaple, HttpSuggestionReasons,
};
use zwipe_ui::generation::Generation;
use zwipe_ui::playstyle::{self, StapleColor};
use zwipe_ui::screens::playstyle::{
    ADDED, COLD_START, LOAD_FAILED, NO_STAPLES, NO_SUGGESTIONS, SHARES_TAGS, SKIPPED, SKIPS_CLEARED,
};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::widgets::toast::Toast;
use crate::outbound::account::SessionFile;
use crate::test_support::{card, create_opened_with, render, screen_text, user};

/// The app's copy for this hint, as it reads.
const HINT: &str = "Your colors, card roles and top oracle tags, read from every deck you own in any format\n• Create deck begins a deck with that commander\n• Maybeboard saves it to your commander maybeboard\n• Remove hides it for a week and pulls the next one up\n• Swipe deals the same ranking: right starts a deck, up saves, left skips it\n• Show narrows the list to one color identity\nBelow them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking";

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
        ticket: load(&harness),
        found: found(held()),
    });
    view_mut(&mut harness).map_opened = Instant::now().checked_sub(Duration::from_secs(5));
    harness
}

/// The newest load's ticket, cleared or not.
fn load(harness: &Harness) -> Ticket {
    load_ticket(harness.sent())
}

/// The newest maybeboard, dismiss or clear's ticket, cleared or not.
fn edit(harness: &Harness) -> Ticket {
    sent_ticket(harness.sent(), |r| match r {
        Request::Maybeboard { ticket, .. }
        | Request::Dismiss { ticket, .. }
        | Request::ClearSkips { ticket } => Some(*ticket),
        _ => None,
    })
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

/// Sends `action` through Playstyle's dispatch, as a key or a closing screen would.
fn act(harness: &mut Harness, action: Action) {
    harness.on_top(|screen, cx| {
        let Screen::Playstyle(view) = screen else {
            panic!("not Playstyle");
        };
        dispatch(view, action, cx);
    });
}

fn press(harness: &mut Harness, codes: &[KeyCode]) {
    for &code in codes {
        harness.press(code);
    }
}

fn names(harness: &Harness) -> Vec<String> {
    view(harness)
        .model
        .rows()
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
    view(harness).model.rows()[index]
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
            screens::Request::MarkHintShown(HINT_SUGGESTED_COMMANDERS.to_owned()),
            Request::Load {
                ticket: load(&harness)
            }
            .into(),
            Request::DeckProfiles.into(),
            oracle_tags::Request::Catalog.into(),
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, "Playstyle");
    assert_eq!(
        hint::text(&hint.content),
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
        "> ▾ Color map [?]",
        "██",
        "Colorless 9",
        "▸ Summary",
        "▸ Card roles",
        "▸ Oracle tags",
        " Show   All   U   R   Azorius WU ",
        "Suggested commanders [?]",
        "▸ 1 Red 00",
        "▸ 1 Red 09",
        " Type   All   Creature   Artifact   Instant ",
        " Color   All   R   Colorless ",
        "Staples [?]",
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
        .collect::<String>()
        .replace('\u{a0}', " ")
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
fn a_stale_or_repeated_load_changes_nothing() {
    let mut harness = loaded();
    let first = load(&harness);
    act(&mut harness, Action::SwipeClosed);
    let second = load(&harness);
    assert_ne!(first, second, "closing the swipe loads again");
    assert_eq!(harness.pending(), [Request::Load { ticket: second }.into()]);
    harness.apply(Update::Loaded {
        ticket: first,
        found: found(vec![suggestion("Stale", &[Green])]),
    });
    assert_ne!(names(&harness), ["Stale"], "the older reply is dropped");
    harness.apply(Update::Loaded {
        ticket: second,
        found: found(vec![suggestion("Fresh", &[Green])]),
    });
    assert_eq!(names(&harness), ["Fresh"]);
    harness.apply(Update::Loaded {
        ticket: second,
        found: found(vec![suggestion("Again", &[Green])]),
    });
    assert_eq!(names(&harness), ["Fresh"], "a repeated reply is dropped");
}

#[test]
fn a_failed_first_load_says_so_and_a_failed_refetch_keeps_the_rows() {
    let mut harness = on_playstyle(true);
    harness.apply(Update::Failed {
        ticket: load(&harness),
        message: "The server is down".to_owned(),
    });
    assert_eq!(view(&harness).model.status(), Status::Failed);
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
    act(&mut harness, Action::SwipeClosed);
    harness.apply(Update::Failed {
        ticket: load(&harness),
        message: "Offline".to_owned(),
    });
    assert_eq!(
        view(&harness).model.status(),
        Status::Loaded,
        "rows on screen stay"
    );
    assert_eq!(names(&harness).len(), 10);
}

#[test]
fn no_decks_is_a_cold_start() {
    let mut harness = on_playstyle(true);
    harness.apply(Update::Loaded {
        ticket: load(&harness),
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
        ticket: load(&harness),
        found: found(Vec::new()),
    });
    assert!(render_text(&harness, 110, 90).contains(NO_SUGGESTIONS));
}

#[test]
fn the_show_row_narrows_the_list_and_its_picked_chip_returns_to_all() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Show;
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    assert_eq!(view(&harness).model.identity(), Some(&[Blue][..]));
    assert_eq!(names(&harness), ["Talrand"]);
    press(&mut harness, &[KeyCode::Enter]);
    assert_eq!(
        view(&harness).model.identity(),
        None,
        "the picked chip again is All"
    );
    assert_eq!(view(&harness).show_cursor, 0);
    assert_eq!(names(&harness).len(), 10);
    press(&mut harness, &[KeyCode::Char('l'), KeyCode::Enter]);
    let talrand = oracle_of(&harness, 0);
    act(&mut harness, Action::Dismiss(talrand));
    harness.apply(Update::Dismissed {
        ticket: edit(&harness),
    });
    assert_eq!(
        view(&harness).model.identities()[0],
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
    assert_eq!(
        harness.pending(),
        [Request::Maybeboard {
            ticket: edit(&harness),
            oracle_id: first
        }
        .into()]
    );
    let save = edit(&harness);
    harness.clear();
    harness.apply(Update::Saved { ticket: save });
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
    assert_eq!(
        harness.pending(),
        [Request::Dismiss {
            ticket: edit(&harness),
            oracle_id: second
        }
        .into()]
    );
    let dismiss = edit(&harness);
    harness.clear();
    harness.apply(Update::Dismissed { ticket: dismiss });
    assert_eq!(
        toasts(&harness).last(),
        Some(&(ToastKind::Info, SKIPPED.to_owned()))
    );
    assert_eq!(names(&harness).len(), 10);
    assert_eq!(view(&harness).model.entries().len(), 12);
    let shown = names(&harness);
    let count = toasts(&harness).len();
    harness.apply(Update::Dismissed { ticket: dismiss });
    harness.apply(Update::Saved { ticket: save });
    assert_eq!(names(&harness), shown, "a repeated reply drops nothing");
    assert_eq!(toasts(&harness).len(), count, "and raises nothing");
}

#[test]
fn a_drop_refetches_when_the_list_runs_short_or_a_load_is_out() {
    let mut harness = loaded();
    act(&mut harness, Action::SwipeClosed);
    let out = load(&harness);
    let first = oracle_of(&harness, 0);
    act(&mut harness, Action::Dismiss(first));
    harness.clear();
    harness.apply(Update::Dismissed {
        ticket: edit(&harness),
    });
    let again = load(&harness);
    assert_ne!(
        again, out,
        "a load from before the drop would bring the row back"
    );
    assert_eq!(harness.pending(), [Request::Load { ticket: again }.into()]);
    harness.clear();
    harness.apply(Update::Loaded {
        ticket: again,
        found: found(held()[1..].to_vec()),
    });
    act(&mut harness, Action::ToggleIdentity(vec![Red]));
    let red = oracle_of(&harness, 0);
    act(&mut harness, Action::Maybeboard(red));
    harness.clear();
    harness.apply(Update::Saved {
        ticket: edit(&harness),
    });
    assert_eq!(harness.pending(), [], "ten red left is a full list");
    let red = oracle_of(&harness, 0);
    act(&mut harness, Action::Maybeboard(red));
    harness.clear();
    harness.apply(Update::Saved {
        ticket: edit(&harness),
    });
    assert_eq!(
        harness.pending(),
        [Request::Load {
            ticket: load(&harness)
        }
        .into()],
        "nine red left is short"
    );
    harness.clear();
    act(&mut harness, Action::Maybeboard(red));
    harness.apply(Update::Failed {
        ticket: edit(&harness),
        message: "Nope".to_owned(),
    });
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
    assert_eq!(create_opened_with(&harness), Some("Red 00"));

    let mut harness = on_playstyle(false);
    harness.apply(Update::Loaded {
        ticket: load(&harness),
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
    assert_eq!(view(&harness).model.staple_type(), Some(CardType::Creature));
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
    assert_eq!(
        view(&harness).model.staple_color(),
        Some(StapleColor::Colorless)
    );
    assert_eq!(view(&harness).matching_staples(), Vec::<HttpStaple>::new());
    assert!(render_text(&harness, 110, 90).contains(NO_STAPLES));
    press(&mut harness, &[KeyCode::Enter]);
    assert_eq!(view(&harness).model.staple_color(), None, "again is All");
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
    assert_eq!(
        harness.pending(),
        [Request::ClearSkips {
            ticket: edit(&harness)
        }
        .into()]
    );
    harness.clear();
    harness.apply(Update::Cleared {
        ticket: edit(&harness),
    });
    assert_eq!(
        toasts(&harness).last(),
        Some(&(ToastKind::Info, SKIPS_CLEARED.to_owned()))
    );
    assert_eq!(
        harness.pending(),
        [Request::Load {
            ticket: load(&harness)
        }
        .into()]
    );
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
        Some(Overlay::Info(topic(Section::Tags).unwrap()))
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
        harness
            .app
            .shared
            .hint
            .as_ref()
            .map(|h| hint::text(&h.content)),
        Some(HINT.to_owned())
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
    assert_eq!(examples.model.slug(), "mana-rock");
    harness.press(KeyCode::Esc);

    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(
        &mut harness,
        &[KeyCode::Enter, KeyCode::Char('j'), KeyCode::Char('j')],
    );
    let table = render_text(&harness, 110, 90);
    assert!(table.contains("Tag"), "{table}");
    assert!(table.contains("Decks [?]"), "{table}");
    assert!(table.contains("Cards [?]"), "{table}");
    assert!(table.contains("Impact [?]"), "{table}");
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
    assert!(printings.model.shows_save(), "pick mode");
    let mut printing = colored("Red 00", &[Red]);
    printing.scryfall_data.oracle_id = Some(oracle_id);
    harness.app.shared.caches.handoff.printing = Some(printing.clone());
    harness.press(KeyCode::Esc);
    let v = view(&harness);
    assert_eq!(v.model.entries()[0].card, printing);
    assert_eq!(v.rows.expanded(), Some(printing.scryfall_data.id));
}

#[test]
fn a_printing_picked_on_a_staple_swaps_into_it_until_the_next_load() {
    let mut harness = loaded();
    view_mut(&mut harness).focus = Focus::Staples;
    view_mut(&mut harness).staple_rows.set_focused(0);
    harness.press(KeyCode::Enter);
    let staple = view(&harness).matching_staples()[0].card.clone();
    let mut printing = card("Sol Ring");
    printing.scryfall_data.oracle_id = staple.scryfall_data.oracle_id;
    harness.app.shared.caches.handoff.printing = Some(printing.clone());
    harness.on_top(Screen::on_return);
    let v = view(&harness);
    assert_eq!(v.matching_staples()[0].card, printing);
    assert_eq!(v.staple_rows.expanded(), Some(printing.scryfall_data.id));
    let same = Box::new(HttpCommanderSuggestions {
        suggestions: held(),
        deck_count: 4,
        profile: v.model.profile().cloned(),
    });
    act(&mut harness, Action::SwipeClosed);
    harness.apply(Update::Loaded {
        ticket: load(&harness),
        found: same,
    });
    assert_eq!(view(&harness).matching_staples()[0].card, staple);
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
    let file = SessionFile::at(std::env::temp_dir().join("zwiper-tui-tests-no-session.json"));
    let account = Arc::new(Account::new(client, file));
    for request in [
        Request::Load {
            ticket: Generation::default().next_ticket(),
        },
        Request::DeckProfiles,
        Request::ClearSkips {
            ticket: Generation::default().next_ticket(),
        },
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
        ticket: load(&harness),
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

#[test]
fn playstyle_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&on_playstyle(true).app, "Esc Back");
    let mut harness = loaded();
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc Back");
    expand_first(&mut harness);
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc Back");
    for (width, height) in crate::test_support::NARROW {
        let screen = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            screen.contains("i Image"),
            "the open row keeps its keys in view at {width}x{height}:\n{screen}"
        );
    }
}

#[test]
fn a_short_terminal_shrinks_the_map_so_every_show_chip_is_in_view() {
    use Color::Black;
    let identities: [&[Color]; 13] = [
        &[White],
        &[Blue],
        &[Black],
        &[Red],
        &[White, Blue],
        &[White, Black],
        &[White, Red],
        &[Blue, Red],
        &[Black, Red],
        &[White, Blue, Black],
        &[White, Blue, Red],
        &[White, Black, Red],
        &[Blue, Red, Green],
    ];
    let mut harness = on_playstyle(true);
    let suggestions = identities
        .iter()
        .enumerate()
        .map(|(i, colors)| suggestion(&format!("Commander {i:02}"), colors))
        .collect();
    harness.apply(Update::Loaded {
        ticket: load(&harness),
        found: found(suggestions),
    });
    view_mut(&mut harness).map_opened = Instant::now().checked_sub(Duration::from_secs(5));
    let last = playstyle::identity_label(&[Blue, Red, Green]).unwrap();
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        let text = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(text.contains("Show"), "at {width}x{height}:\n{text}");
        assert!(
            text.contains(last),
            "the last Show chip at {width}x{height}:\n{text}"
        );
    }
}

#[test]
fn the_tag_tables_column_markers_are_buttons_and_the_picked_one_lights() {
    let mut harness = on_playstyle(true);
    let mut reply = found(held());
    if let Some(profile) = reply.profile.as_mut() {
        profile.tags.push(HttpProfileTag {
            slug: "repeatable-creature-tokens".to_owned(),
            role: None,
            decks: 1,
            cards: 3,
            score: 0.5,
        });
    }
    harness.apply(Update::Loaded {
        ticket: load(&harness),
        found: reply,
    });
    view_mut(&mut harness).map_opened = Instant::now().checked_sub(Duration::from_secs(5));
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    view_mut(&mut harness).focus = Focus::Header(Section::Tags);
    press(&mut harness, &[KeyCode::Enter, KeyCode::Char('j')]);
    assert_eq!(view(&harness).focus, Focus::TagHead);
    view_mut(&mut harness).column = super::Column::Cards;
    for (width, height) in crate::test_support::BUTTON_SIZES {
        let rows = crate::test_support::rows(&harness.app, width, height);
        let bare = crate::test_support::bare_about_markers(&rows);
        assert!(bare.is_empty(), "bare ? at {bare:?}:\n{}", rows.join("\n"));
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        for (label, lit) in [("Decks", false), ("Cards", true), ("Impact", false)] {
            let context = format!("{label} at {width}x{height}");
            let found = crate::test_support::find_all(buffer, &format!("{label} [?]"));
            let &(x, y) = found
                .first()
                .unwrap_or_else(|| panic!("no {context}:\n{}", rows.join("\n")));
            let width = u16::try_from(label.len()).unwrap();
            for c in x..x + width {
                let cell = &buffer[(c, y)];
                assert!(
                    !cell.modifier.contains(ratatui::style::Modifier::REVERSED),
                    "{context}: the label is never filled"
                );
                assert_eq!(
                    cell.modifier.contains(ratatui::style::Modifier::BOLD),
                    lit,
                    "{context}: bold only while picked"
                );
                if lit {
                    assert_eq!(Some(cell.fg), palette.accent.fg, "{context}: accent");
                }
            }
            let at = x + width + 1;
            crate::test_support::assert_about_marker(buffer, &palette, (at, y), lit, &context);
        }
        let at = rows.iter().position(|r| r.contains("Decks [?]")).unwrap();
        let header = &rows[at];
        assert!(
            header.contains("Decks [?] Cards [?] Impact [?]"),
            "{header}"
        );
        assert!(
            !rows[at + 1].trim_start().starts_with("[?]"),
            "the header never wraps at {width}x{height}:\n{}",
            rows.join("\n")
        );
        let decks = header[..header.find("Decks").unwrap()].chars().count();
        for row in rows.iter().skip(at + 1).take(3) {
            let digit = row.find(|c: char| c.is_ascii_digit()).unwrap();
            assert_eq!(
                row[..digit].chars().count(),
                decks + 1,
                "the numbers sit under their column at {width}x{height}: {row}"
            );
        }
    }
}
