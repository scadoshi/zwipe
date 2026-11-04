use zwipe_core::domain::card::scryfall_data::colors::Color::{Blue, Green, Red, White};
use zwipe_core::http::contracts::suggestions::{
    HttpColorAffinity, HttpColorCards, HttpColorIdentityCount, HttpColorMap, HttpProfileRole,
    HttpProfileTag, HttpSharedTag,
};

use super::*;
use crate::deck::VERIFY_TO_CREATE;
use crate::test_support::card;
use crate::toast::{LONG, QUICK};

/// A suggestion named `name` with `identity` and no reasons.
fn suggestion(name: &str, identity: &[Color]) -> HttpCommanderSuggestion {
    let mut card = card(name);
    card.scryfall_data.color_identity = identity.iter().copied().collect();
    HttpCommanderSuggestion {
        card,
        reasons: HttpSuggestionReasons {
            tags: Vec::new(),
            colors: Vec::new(),
        },
    }
}

/// A staple named `name` with `type_line` and `identity`, in `decks` decks.
fn staple(name: &str, type_line: &str, identity: &[Color], decks: u32) -> HttpStaple {
    let mut card = card(name);
    card.scryfall_data.type_line = Some(type_line.to_owned());
    card.scryfall_data.color_identity = identity.iter().copied().collect();
    HttpStaple { card, decks }
}

fn oracle(suggestion: &HttpCommanderSuggestion) -> Uuid {
    suggestion.card.scryfall_data.oracle_id.unwrap()
}

fn names(suggestions: &[HttpCommanderSuggestion]) -> Vec<String> {
    suggestions
        .iter()
        .map(|s| s.card.scryfall_data.name.clone())
        .collect()
}

fn staple_names(staples: &[HttpStaple]) -> Vec<String> {
    staples
        .iter()
        .map(|s| s.card.scryfall_data.name.clone())
        .collect()
}

/// A profile over `deck_count` decks with no colors, roles, tags, staples or map.
fn bare_profile(deck_count: u32) -> HttpBuilderProfile {
    HttpBuilderProfile {
        deck_count,
        colors: Vec::new(),
        top_identity: None,
        tags: Vec::new(),
        roles: Vec::new(),
        staples: Vec::new(),
        color_map: None,
    }
}

/// A full profile over 4 decks: blue in 3, Azorius in 2, one ramp role with one tag, a color map with blue cards, and three staples.
fn full_profile() -> HttpBuilderProfile {
    HttpBuilderProfile {
        deck_count: 4,
        colors: vec![HttpColorAffinity {
            color: "U".to_owned(),
            decks: 3,
        }],
        top_identity: Some(HttpColorIdentityCount {
            colors: vec!["U".to_owned(), "W".to_owned()],
            decks: 2,
        }),
        tags: vec![HttpProfileTag {
            slug: "mana-rock".to_owned(),
            role: Some("ramp".to_owned()),
            decks: 3,
            cards: 7,
            score: 1.5,
        }],
        roles: vec![HttpProfileRole {
            role: "ramp".to_owned(),
            decks: 3,
        }],
        staples: vec![
            staple("Sol Ring", "Artifact", &[], 4),
            staple("Counterspell", "Instant", &[Blue], 3),
            staple("Llanowar Elves", "Creature — Elf Druid", &[Green], 2),
        ],
        color_map: Some(HttpColorMap {
            colors: vec![HttpColorCards {
                color: "U".to_owned(),
                cards: 12,
            }],
            colorless_cards: 0,
            pairs: Vec::new(),
        }),
    }
}

fn found(
    suggestions: Vec<HttpCommanderSuggestion>,
    profile: Option<HttpBuilderProfile>,
) -> Box<HttpCommanderSuggestions> {
    let deck_count = profile.as_ref().map_or(0, |p| p.deck_count);
    Box::new(HttpCommanderSuggestions {
        suggestions,
        deck_count,
        profile,
    })
}

/// A state whose first load is out, with the ticket it holds.
fn loading() -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load);
    let [
        Effect::OfferHint(OneTimeHint::SuggestedCommanders),
        Effect::FetchSuggestions(ticket),
        Effect::FetchDeckProfiles,
    ] = effects.as_slice()
    else {
        panic!("Load offers the hint and fetches: {effects:?}");
    };
    (state, *ticket)
}

/// A state with `suggestions` and `profile` loaded.
fn loaded(suggestions: Vec<HttpCommanderSuggestion>, profile: HttpBuilderProfile) -> State {
    let (mut state, ticket) = loading();
    let found = found(suggestions, Some(profile));
    assert_eq!(state.update(Action::Loaded { ticket, found }), []);
    state
}

/// Krenko (R), Narset (WUR), Zada (R) and Kozilek (colorless), best fit first.
fn ranking() -> Vec<HttpCommanderSuggestion> {
    vec![
        suggestion("Krenko", &[Red]),
        suggestion("Narset", &[White, Blue, Red]),
        suggestion("Zada", &[Red]),
        suggestion("Kozilek", &[]),
    ]
}

/// `n` mono-red suggestions named r0, r1, ...
fn reds(n: usize) -> Vec<HttpCommanderSuggestion> {
    (0..n)
        .map(|i| suggestion(&format!("r{i}"), &[Red]))
        .collect()
}

/// The ticket of the one request `effects` sends.
fn ticket_of(effects: &[Effect]) -> Ticket {
    match effects {
        [
            Effect::AddToMaybeboard { ticket, .. }
            | Effect::DismissCommander { ticket, .. }
            | Effect::ClearDismissals(ticket),
        ] => *ticket,
        _ => panic!("one request: {effects:?}"),
    }
}

fn fetch_ticket(effect: &Effect) -> Ticket {
    let Effect::FetchSuggestions(ticket) = effect else {
        panic!("a fetch: {effect:?}");
    };
    *ticket
}

#[test]
fn a_new_screen_is_loading_with_every_chip_on_all() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.message(), None);
    assert!(!state.shows_swipe(), "no Swipe until a deck is counted");
    assert_eq!(state.sections(), []);
    assert_eq!(state.identity(), None);
    assert_eq!(state.staple_type(), None);
    assert_eq!(state.staple_color(), None);
    assert!(!state.shows_staples());
}

#[test]
fn load_offers_the_hint_once_and_fetches_every_time() {
    let (mut state, _) = loading();
    let effects = state.update(Action::Load);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchSuggestions(_), Effect::FetchDeckProfiles]
        ),
        "a second Load does not offer the hint again: {effects:?}"
    );
}

#[test]
fn a_landed_load_shows_the_ranking_and_the_profile() {
    let state = loaded(ranking(), full_profile());
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.message(), None);
    assert!(state.shows_swipe());
    assert_eq!(state.deck_count(), 4);
    assert_eq!(
        names(&state.rows()),
        ["Krenko", "Narset", "Zada", "Kozilek"]
    );
    assert_eq!(state.list_message(), None);
    assert_eq!(state.entries().len(), 4);
    assert!(state.profile().is_some());
}

#[test]
fn a_load_with_no_deck_is_the_cold_start() {
    let (mut state, ticket) = loading();
    let _ = state.update(Action::Loaded {
        ticket,
        found: found(Vec::new(), None),
    });
    assert_eq!(state.status(), Status::ColdStart);
    assert_eq!(state.message(), Some(COLD_START));
    assert!(!state.shows_swipe());
    assert_eq!(state.sections(), []);
    assert!(!state.shows_staples());
}

#[test]
fn a_failed_first_load_says_so() {
    let (mut state, ticket) = loading();
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.message(), Some(LOAD_FAILED));
}

#[test]
fn a_failed_refetch_keeps_the_held_ranking() {
    let mut state = loaded(ranking(), full_profile());
    let effects = state.update(Action::SwipeClosed);
    let ticket = fetch_ticket(&effects[0]);
    let _ = state.update(Action::Failed { ticket });
    assert_eq!(state.status(), Status::Loaded, "the rows stay up");
    assert_eq!(state.entries().len(), 4);
}

#[test]
fn a_landed_load_clears_a_failure() {
    let (mut state, ticket) = loading();
    let _ = state.update(Action::Failed { ticket });
    let effects = state.update(Action::SwipeClosed);
    let ticket = fetch_ticket(&effects[0]);
    let _ = state.update(Action::Loaded {
        ticket,
        found: found(ranking(), Some(full_profile())),
    });
    assert_eq!(state.status(), Status::Loaded);
}

#[test]
fn a_late_load_reply_is_ignored() {
    let (mut state, first) = loading();
    let effects = state.update(Action::SwipeClosed);
    let second = fetch_ticket(&effects[0]);
    let _ = state.update(Action::Loaded {
        ticket: first,
        found: found(reds(3), Some(full_profile())),
    });
    assert_eq!(state.status(), Status::Loading, "the older reply landed");
    let _ = state.update(Action::Failed { ticket: first });
    assert_eq!(state.status(), Status::Loading, "the older failure landed");
    let _ = state.update(Action::Loaded {
        ticket: second,
        found: found(ranking(), Some(full_profile())),
    });
    assert_eq!(
        names(&state.rows()).first().map(String::as_str),
        Some("Krenko")
    );
    let _ = state.update(Action::Loaded {
        ticket: second,
        found: found(reds(3), Some(full_profile())),
    });
    assert_eq!(
        names(&state.rows()).first().map(String::as_str),
        Some("Krenko"),
        "a reply lands once"
    );
}

#[test]
fn an_empty_ranking_says_no_suggestions() {
    let state = loaded(Vec::new(), full_profile());
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.list_message().as_deref(), Some(NO_SUGGESTIONS));
}

#[test]
fn the_list_shows_the_first_ten_and_swipe_deals_them_all() {
    let state = loaded(reds(25), full_profile());
    assert_eq!(state.rows().len(), LIST_SIZE);
    assert_eq!(state.filtered().len(), 25);
}

#[test]
fn show_narrows_to_one_identity_and_all_undoes_it() {
    let mut state = loaded(ranking(), full_profile());
    assert_eq!(
        state.identities(),
        vec![vec![Red], vec![White, Blue, Red], vec![]]
    );
    let _ = state.update(Action::ToggleIdentity(vec![Red]));
    assert_eq!(state.identity(), Some([Red].as_slice()));
    assert_eq!(names(&state.rows()), ["Krenko", "Zada"]);
    let _ = state.update(Action::ToggleIdentity(vec![Red]));
    assert_eq!(state.identity(), None, "the picked chip again is All");
    let _ = state.update(Action::ToggleIdentity(vec![]));
    assert_eq!(names(&state.rows()), ["Kozilek"]);
    let _ = state.update(Action::ShowAll);
    assert_eq!(state.rows().len(), 4);
}

#[test]
fn a_show_chip_stays_after_its_last_commander_goes() {
    let mut state = loaded(ranking(), full_profile());
    let _ = state.update(Action::ToggleIdentity(vec![]));
    let kozilek = oracle(&state.entries()[3]);
    let effects = state.update(Action::Dismiss(kozilek));
    let _ = state.update(Action::Dismissed {
        ticket: ticket_of(&effects),
    });
    assert!(state.identities().contains(&vec![]), "the chip went");
    assert_eq!(
        state.list_message().as_deref(),
        Some("No colorless commanders in your top 100")
    );
}

#[test]
fn maybeboard_drops_the_row_and_toasts() {
    let mut state = loaded(reds(12), full_profile());
    let target = oracle(&state.entries()[0]);
    let effects = state.update(Action::Maybeboard(target));
    let [Effect::AddToMaybeboard { ticket, oracle_id }] = effects.as_slice() else {
        panic!("a save: {effects:?}");
    };
    assert_eq!(*oracle_id, target);
    assert_eq!(state.entries().len(), 12, "the row waits for the server");
    let effects = state.update(Action::Saved { ticket: *ticket });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(ToastKind::Success, ADDED, QUICK))],
        "eleven left, so no refetch"
    );
    assert_eq!(state.entries().len(), 11);
    assert_eq!(names(&state.rows()).first().map(String::as_str), Some("r1"));
    assert_eq!(state.rows().len(), LIST_SIZE, "the next one moves up");
}

#[test]
fn dismiss_drops_the_row_and_toasts() {
    let mut state = loaded(reds(12), full_profile());
    let target = oracle(&state.entries()[2]);
    let effects = state.update(Action::Dismiss(target));
    let [Effect::DismissCommander { ticket, oracle_id }] = effects.as_slice() else {
        panic!("a skip: {effects:?}");
    };
    assert_eq!(*oracle_id, target);
    let effects = state.update(Action::Dismissed { ticket: *ticket });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(ToastKind::Info, SKIPPED, QUICK))]
    );
    assert!(!names(state.entries()).contains(&"r2".to_owned()));
}

#[test]
fn a_drop_that_leaves_under_ten_refetches_first() {
    let mut state = loaded(reds(10), full_profile());
    let effects = state.update(Action::Dismiss(oracle(&state.entries()[0])));
    let effects = state.update(Action::Dismissed {
        ticket: ticket_of(&effects),
    });
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchSuggestions(_), Effect::Toast(_)]
        ),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::Loaded, "the refetch is silent");
}

#[test]
fn a_drop_counts_against_the_show_identity() {
    let mut held = reds(12);
    held.push(suggestion("Talrand", &[Blue]));
    held.push(suggestion("Teferi", &[Blue]));
    let mut state = loaded(held, full_profile());
    let _ = state.update(Action::ToggleIdentity(vec![Blue]));
    let talrand = oracle(&state.rows()[0]);
    let effects = state.update(Action::Dismiss(talrand));
    let effects = state.update(Action::Dismissed {
        ticket: ticket_of(&effects),
    });
    assert!(
        matches!(effects.first(), Some(Effect::FetchSuggestions(_))),
        "one blue left is under ten: {effects:?}"
    );
}

#[test]
fn a_drop_while_a_load_is_out_sends_it_again() {
    let mut state = loaded(reds(15), full_profile());
    let effects = state.update(Action::SwipeClosed);
    let outdated = fetch_ticket(&effects[0]);
    let effects = state.update(Action::Maybeboard(oracle(&state.entries()[0])));
    let effects = state.update(Action::Saved {
        ticket: ticket_of(&effects),
    });
    let fresh = fetch_ticket(&effects[0]);
    assert_ne!(fresh, outdated);
    let _ = state.update(Action::Loaded {
        ticket: outdated,
        found: found(reds(15), Some(full_profile())),
    });
    assert_eq!(
        state.entries().len(),
        14,
        "the outdated load brought the row back"
    );
}

#[test]
fn a_failed_edit_changes_nothing_and_its_late_reply_is_ignored() {
    let mut state = loaded(reds(12), full_profile());
    let effects = state.update(Action::Dismiss(oracle(&state.entries()[0])));
    let ticket = ticket_of(&effects);
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert_eq!(
        state.status(),
        Status::Loaded,
        "an edit failure is not a load failure"
    );
    assert_eq!(state.update(Action::Dismissed { ticket }), []);
    assert_eq!(state.entries().len(), 12);
}

#[test]
fn a_reply_of_the_wrong_kind_is_ignored() {
    let mut state = loaded(reds(12), full_profile());
    let effects = state.update(Action::Dismiss(oracle(&state.entries()[0])));
    let ticket = ticket_of(&effects);
    assert_eq!(state.update(Action::Saved { ticket }), []);
    assert_eq!(state.update(Action::Cleared { ticket }), []);
    assert_eq!(state.entries().len(), 12);
    assert_ne!(state.update(Action::Dismissed { ticket }), []);
    assert_eq!(state.entries().len(), 11);
}

#[test]
fn edits_land_in_any_order() {
    let mut state = loaded(reds(15), full_profile());
    let first = ticket_of(&state.update(Action::Dismiss(oracle(&state.entries()[0]))));
    let second = ticket_of(&state.update(Action::Maybeboard(oracle(&state.entries()[1]))));
    let _ = state.update(Action::Saved { ticket: second });
    let _ = state.update(Action::Dismissed { ticket: first });
    assert_eq!(state.entries().len(), 13);
}

#[test]
fn clear_skips_toasts_then_refetches() {
    let mut state = loaded(ranking(), full_profile());
    let effects = state.update(Action::ClearSkips);
    let ticket = ticket_of(&effects);
    let effects = state.update(Action::Cleared { ticket });
    let [Effect::Toast(toast), Effect::FetchSuggestions(_)] = effects.as_slice() else {
        panic!("a toast and a fetch: {effects:?}");
    };
    assert_eq!(*toast, Toast::new(ToastKind::Info, SKIPS_CLEARED, QUICK));
    assert_eq!(state.update(Action::Cleared { ticket }), [], "lands once");
}

#[test]
fn swipe_closed_refetches_silently() {
    let mut state = loaded(ranking(), full_profile());
    let effects = state.update(Action::SwipeClosed);
    assert!(matches!(effects.as_slice(), [Effect::FetchSuggestions(_)]));
    assert_eq!(state.status(), Status::Loaded);
}

#[test]
fn create_seeds_the_commander_and_opens_create() {
    let mut state = loaded(ranking(), full_profile());
    let card = Box::new(ranking()[0].card.clone());
    let effects = state.update(Action::Create {
        card: card.clone(),
        verified: true,
    });
    assert_eq!(
        effects,
        [
            Effect::SeedCommander(card),
            Effect::Navigate(Destination::CreateDeck)
        ]
    );
}

#[test]
fn create_is_gated_for_an_unverified_account_with_a_deck() {
    let mut state = loaded(ranking(), full_profile());
    let card = ranking()[0].card.clone();
    let unverified = |card: &Card| Action::Create {
        card: Box::new(card.clone()),
        verified: false,
    };
    assert!(
        matches!(
            state.update(unverified(&card)).as_slice(),
            [Effect::SeedCommander(_), Effect::Navigate(_)]
        ),
        "the deck count is still out, so the gate counts none"
    );
    let _ = state.update(Action::DeckProfilesLoaded { count: 1 });
    assert_eq!(
        state.update(unverified(&card)),
        [Effect::Toast(Toast::new(
            ToastKind::Warning,
            VERIFY_TO_CREATE,
            LONG
        ))]
    );
}

#[test]
fn pick_printing_swaps_the_suggestion_sharing_its_oracle() {
    let mut state = loaded(ranking(), full_profile());
    let krenko = state.entries()[0].card.clone();
    let mut printing = card("Krenko, Mob Boss");
    printing.scryfall_data.oracle_id = krenko.scryfall_data.oracle_id;
    let _ = state.update(Action::PickPrinting(Box::new(printing.clone())));
    assert_eq!(state.entries()[0].card, printing);

    let mut stray = card("Sol Ring");
    stray.scryfall_data.oracle_id = None;
    let before = state.clone();
    let _ = state.update(Action::PickPrinting(Box::new(stray)));
    assert_eq!(state, before, "no oracle id changes nothing");
}

#[test]
fn back_goes_to_the_screen_before() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn the_sections_follow_the_profile() {
    let state = loaded(ranking(), full_profile());
    assert_eq!(state.sections(), Section::ALL);
    assert!(state.color_map().is_some());
    let summary = state.summary().unwrap();
    assert_eq!(summary.decks, "4");
    assert_eq!(state.role_bars().len(), 1);
    assert_eq!(state.role_tags("ramp"), ["mana-rock"]);
    assert_eq!(state.role_tags("draw"), Vec::<String>::new());
    assert_eq!(state.tag_rows().len(), 1);

    let state = loaded(ranking(), bare_profile(2));
    assert_eq!(
        state.sections(),
        [Section::ColorMap, Section::Summary],
        "no roles or tags, no chart or table"
    );
    assert_eq!(state.color_map(), None, "an empty map draws Home's line");
}

#[test]
fn only_the_color_map_opens_on_entry() {
    let open: Vec<Section> = Section::ALL
        .into_iter()
        .filter(|s| s.opens_on_entry())
        .collect();
    assert_eq!(open, [Section::ColorMap]);
}

#[test]
fn the_staple_rows_combine_and_reset_when_the_panel_goes() {
    let mut state = loaded(ranking(), full_profile());
    assert!(state.shows_staples());
    assert_eq!(
        state.staple_types(),
        [CardType::Creature, CardType::Artifact, CardType::Instant]
    );
    assert_eq!(
        state.staple_colors(),
        [
            StapleColor::Of(Blue),
            StapleColor::Of(Green),
            StapleColor::Colorless
        ]
    );
    let _ = state.update(Action::ToggleType(CardType::Instant));
    assert_eq!(staple_names(&state.matching_staples()), ["Counterspell"]);
    let _ = state.update(Action::ToggleColor(StapleColor::Of(Green)));
    assert_eq!(state.matching_staples(), []);
    assert_eq!(state.staples_message(), Some(NO_STAPLES));
    let _ = state.update(Action::ToggleColor(StapleColor::Of(Green)));
    assert_eq!(state.staple_color(), None, "the picked chip again is All");
    let _ = state.update(Action::ToggleType(CardType::Artifact));
    assert_eq!(state.staple_type(), Some(CardType::Artifact));
    let _ = state.update(Action::TypeAll);
    let _ = state.update(Action::ColorAll);
    assert_eq!(state.matching_staples().len(), 3);
    assert_eq!(state.staples_message(), None);

    let _ = state.update(Action::ToggleType(CardType::Instant));
    let ticket = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let _ = state.update(Action::Loaded {
        ticket,
        found: found(ranking(), Some(full_profile())),
    });
    assert_eq!(
        state.staple_type(),
        Some(CardType::Instant),
        "a refetch keeps the panel and its pick"
    );
    let ticket = fetch_ticket(&state.update(Action::SwipeClosed)[0]);
    let _ = state.update(Action::Loaded {
        ticket,
        found: found(ranking(), Some(bare_profile(4))),
    });
    assert!(!state.shows_staples());
    assert_eq!(state.staple_type(), None, "the panel went, so its pick did");
}

#[test]
fn reasons_draw_nothing_without_a_tag_or_color() {
    let none = HttpSuggestionReasons {
        tags: Vec::new(),
        colors: Vec::new(),
    };
    assert_eq!(reasons(&none, 3), None);

    let colors_only = HttpSuggestionReasons {
        tags: Vec::new(),
        colors: vec![HttpColorAffinity {
            color: "R".to_owned(),
            decks: 2,
        }],
    };
    assert_eq!(
        reasons(&colors_only, 3),
        Some(Reasons {
            tags: None,
            colors: vec!["You build in red (2 of your 3 decks)".to_owned()],
        })
    );

    let tags_only = HttpSuggestionReasons {
        tags: vec![HttpSharedTag {
            slug: "treasure".to_owned(),
            role: Some("ramp".to_owned()),
        }],
        colors: vec![HttpColorAffinity {
            color: "C".to_owned(),
            decks: 2,
        }],
    };
    let got = reasons(&tags_only, 3).unwrap();
    assert_eq!(got.tags.unwrap().roles, ["ramp"]);
    assert!(got.colors.is_empty(), "a code outside WUBRG has no line");
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Playstyle");
    assert_eq!(LOAD_FAILED, "Could not load your suggestions");
    assert_eq!(COLD_START, "No playstyle yet");
    assert_eq!(NO_SUGGESTIONS, "No suggestions right now");
    assert_eq!(LIST_TITLE, "Suggested commanders");
    assert_eq!(SUMMARY_TITLE, "Summary");
    assert_eq!(COLORS_TITLE, "Color map");
    assert_eq!(ROLES_TITLE, "Card roles");
    assert_eq!(TAGS_TITLE, "Oracle tags");
    assert_eq!(SUMMARY_DECKS, "Decks");
    assert_eq!(SUMMARY_TOP_COLORS, "Top colors");
    assert_eq!(SUMMARY_TOP_COLOR, "Top color");
    assert_eq!(TAG_COLUMN, "Tag");
    assert_eq!(DECKS_COLUMN, "Decks");
    assert_eq!(CARDS_COLUMN, "Cards");
    assert_eq!(IMPACT_COLUMN, "Impact");
    assert_eq!(STAPLES_TITLE, "Staples");
    assert_eq!(NO_STAPLES, "No staples match");
    assert_eq!(SHOW, "Show");
    assert_eq!(ALL, "All");
    assert_eq!(ART, "Art");
    assert_eq!(TYPE, "Type");
    assert_eq!(COLOR, "Color");
    assert_eq!(WHY, "Why this commander?");
    assert_eq!(SHARES_TAGS, "Shares oracle tags with your decks");
    assert_eq!(CREATE_DECK, "Create deck");
    assert_eq!(MAYBEBOARD, "Maybeboard");
    assert_eq!(REMOVE, "Remove");
    assert_eq!(BACK, "Back");
    assert_eq!(SWIPE, "Swipe");
    assert_eq!(MORE, "More");
    assert_eq!(MORE_TITLE, "More actions");
    assert_eq!(CLEAR_SKIPS, "Clear skips");
    assert_eq!(
        CLEAR_BODY,
        "Commanders you skipped will be suggested again. This can't be undone."
    );
    assert_eq!(CANCEL, "Cancel");
    assert_eq!(CLEAR, "Clear");
    assert_eq!(ADDED, "Added to your commander maybeboard");
    assert_eq!(SKIPPED, "Skipped for a week");
    assert_eq!(SKIPS_CLEARED, "Skips cleared");
    let titles: Vec<&str> = Section::ALL.into_iter().map(Section::title).collect();
    assert_eq!(
        titles,
        ["Color map", "Summary", "Card roles", "Oracle tags"]
    );
}
