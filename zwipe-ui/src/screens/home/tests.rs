use chrono::{Duration, TimeZone};
use zwipe_core::domain::user::{Email, Username};
use zwipe_core::http::contracts::suggestions::{HttpColorCards, HttpColorMap};

use super::*;
use crate::test_support::card;
use crate::toast::ToastKind;

/// 12:34 UTC on a fixed day, mid-hour.
fn noon() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 30, 12, 34, 0).unwrap()
}

fn figures(n: u64) -> Figures {
    Figures {
        decks_created: Some(n),
        searches: Some(n),
        cards_swiped: Some(n),
    }
}

fn held(n: u64, age: Age) -> CachedFigures {
    CachedFigures {
        figures: figures(n),
        age,
    }
}

fn user(verified: bool) -> Box<User> {
    let mut user = User::new(
        Uuid::nil(),
        Username::new("scotty").unwrap(),
        Email::new("me@zwipe.net").unwrap(),
    );
    user.email_verified_at = verified.then(Utc::now);
    Box::new(user)
}

/// A summary with `decks` decks and these cards per color, WUBRG.
fn summary(decks: u32, cards: [u32; 5], colorless: u32) -> HttpPlayStyleSummary {
    let colors = ["W", "U", "B", "R", "G"]
        .iter()
        .zip(cards)
        .map(|(color, cards)| HttpColorCards {
            color: (*color).to_owned(),
            cards,
        })
        .collect();
    HttpPlayStyleSummary {
        deck_count: decks,
        color_map: Some(HttpColorMap {
            colors,
            pairs: Vec::new(),
            colorless_cards: colorless,
        }),
    }
}

/// A flavor card named `name` quoting `text`, fetched at noon.
fn flavor(name: &str, text: Option<&str>) -> FlavorCard {
    let mut card = card(name);
    card.scryfall_data.flavor_text = text.map(str::to_owned);
    FlavorCard::fetched(card, noon())
}

/// A signed-in user's Home with nothing held, at noon.
fn cold() -> Cached {
    Cached {
        user: Some(Uuid::from_u128(7)),
        now: noon(),
        ..Cached::default()
    }
}

/// Everything held and fresh, so Load only asks for the user, the preferences and the summary.
fn warm() -> Cached {
    Cached {
        everybody: Some(held(3, Age::Fresh)),
        mine: Some(held(1, Age::Fresh)),
        play_style: Some(summary(2, [1, 0, 0, 0, 4], 0)),
        flavor: Some(flavor("Llanowar Elves", Some("One bone broken"))),
        decks: Some(vec![deck(1, "Elves"), deck(2, "Goblins")]),
        ..cold()
    }
}

fn load(cached: Cached) -> (State, Vec<Effect>) {
    let mut state = State::new();
    let effects = state.update(Action::Load(Box::new(cached)));
    (state, effects)
}

/// The ticket of the first effect `pick` matches.
fn ticket_of(effects: &[Effect], pick: fn(&Effect) -> Option<Ticket>) -> Ticket {
    effects
        .iter()
        .find_map(pick)
        .unwrap_or_else(|| panic!("no such fetch in {effects:?}"))
}

fn user_ticket(e: &Effect) -> Option<Ticket> {
    match e {
        Effect::FetchUser(t) => Some(*t),
        _ => None,
    }
}

fn counts_ticket(e: &Effect) -> Option<Ticket> {
    match e {
        Effect::FetchCounts(t) => Some(*t),
        _ => None,
    }
}

fn my_counts_ticket(e: &Effect) -> Option<Ticket> {
    match e {
        Effect::FetchMyCounts(t) => Some(*t),
        _ => None,
    }
}

fn play_style_ticket(e: &Effect) -> Option<Ticket> {
    match e {
        Effect::FetchPlayStyle(t) => Some(*t),
        _ => None,
    }
}

fn decks_ticket(e: &Effect) -> Option<Ticket> {
    match e {
        Effect::FetchDecks(t) => Some(*t),
        _ => None,
    }
}

/// A deck tile named `name`, with the id `n`.
fn deck(n: u128, name: &str) -> DeckTile {
    DeckTile {
        id: Uuid::from_u128(n),
        name: name.to_owned(),
        format: None,
        color_identity: Vec::new(),
        art_url: None,
    }
}

fn flavor_ticket(e: &Effect) -> Option<Ticket> {
    match e {
        Effect::FetchFlavor(t) => Some(*t),
        _ => None,
    }
}

#[test]
fn a_new_state_ghosts_every_panel() {
    let state = State::new();
    assert_eq!(state.scope(), Scope::Everybody);
    assert_eq!(state.figures(), Figures::default());
    assert!(!state.live());
    assert_eq!(state.play_style(), PlayStyle::Loading);
    assert_eq!(state.decks(), Decks::Loading);
    assert_eq!(state.flavor(), Flavor::Loading);
    assert_eq!(state.flavor_card(), None);
    assert_eq!(state.buy_links(), []);
}

#[test]
fn a_cold_load_asks_for_everything_then_offers_the_hint() {
    let (state, effects) = load(cold());
    let [
        Effect::FetchUser(_),
        Effect::FetchPreferences,
        Effect::FetchCounts(_),
        Effect::FetchPlayStyle(_),
        Effect::FetchDecks(_),
        Effect::FetchFlavor(_),
        Effect::OfferHint(OneTimeHint::FirstLogin),
    ] = effects.as_slice()
    else {
        panic!("a cold load: {effects:?}");
    };
    assert_eq!(state.figures(), Figures::default(), "the figures roll");
    assert_eq!(state.play_style(), PlayStyle::Loading);
    assert_eq!(state.flavor(), Flavor::Loading);
}

#[test]
fn a_warm_load_draws_what_is_held_and_skips_fresh_copies() {
    let (state, effects) = load(warm());
    let [
        Effect::FetchUser(_),
        Effect::FetchPreferences,
        Effect::FetchPlayStyle(_),
        Effect::FetchDecks(_),
        Effect::OfferHint(OneTimeHint::FirstLogin),
    ] = effects.as_slice()
    else {
        panic!("a warm load: {effects:?}");
    };
    assert_eq!(state.figures(), figures(3));
    assert!(state.live());
    assert_eq!(
        state.play_style(),
        PlayStyle::Bars(ColorMapCounts {
            cards: [1, 0, 0, 0, 4],
            colorless: 0
        })
    );
    assert!(matches!(state.flavor(), Flavor::Quote(_)));
}

#[test]
fn signed_out_home_asks_only_for_what_needs_no_session() {
    let (_, effects) = load(Cached {
        user: None,
        scope: Scope::OnlyMe,
        now: noon(),
        ..Cached::default()
    });
    let [
        Effect::FetchCounts(_),
        Effect::FetchFlavor(_),
        Effect::OfferHint(OneTimeHint::FirstLogin),
    ] = effects.as_slice()
    else {
        panic!("signed out: {effects:?}");
    };
}

#[test]
fn restored_counts_show_without_live_and_are_asked_for_again() {
    let (mut state, effects) = load(Cached {
        everybody: Some(held(5, Age::Restored)),
        ..warm()
    });
    assert_eq!(state.figures(), figures(5));
    assert!(!state.live(), "a copy from the last launch is not live");
    let ticket = ticket_of(&effects, counts_ticket);
    let _ = state.update(Action::CountsLoaded {
        ticket,
        figures: figures(6),
    });
    assert_eq!(state.figures(), figures(6));
    assert!(state.live());
}

#[test]
fn stale_counts_stay_live_while_asked_for_again() {
    let (state, effects) = load(Cached {
        everybody: Some(held(5, Age::Stale)),
        ..warm()
    });
    assert!(state.live());
    assert!(effects.iter().any(|e| counts_ticket(e).is_some()));
}

#[test]
fn only_me_shows_the_users_own_counts_once_they_arrive() {
    let (mut state, effects) = load(Cached {
        scope: Scope::OnlyMe,
        everybody: Some(held(9, Age::Fresh)),
        ..cold()
    });
    assert_eq!(state.figures(), Figures::default(), "mine roll");
    assert!(!state.live());
    let ticket = ticket_of(&effects, my_counts_ticket);
    let _ = state.update(Action::MyCountsLoaded {
        ticket,
        figures: figures(2),
    });
    assert_eq!(state.figures(), figures(2));
    assert!(state.live());
}

#[test]
fn toggling_saves_the_scope_and_asks_for_mine_unless_fresh() {
    let (mut state, _) = load(cold());
    let effects = state.update(Action::ToggleScope { mine_fresh: false });
    let [Effect::SaveScope(Scope::OnlyMe), Effect::FetchMyCounts(_)] = effects.as_slice() else {
        panic!("to only me: {effects:?}");
    };
    assert_eq!(state.scope(), Scope::OnlyMe);

    assert_eq!(
        state.update(Action::ToggleScope { mine_fresh: false }),
        [Effect::SaveScope(Scope::Everybody)],
        "everybody's counts are not asked for on a switch"
    );
    assert_eq!(
        state.update(Action::ToggleScope { mine_fresh: true }),
        [Effect::SaveScope(Scope::OnlyMe)],
        "a fresh copy of mine is enough"
    );
}

#[test]
fn toggling_signed_out_asks_for_nothing() {
    let (mut state, _) = load(Cached {
        user: None,
        ..Cached::default()
    });
    assert_eq!(
        state.update(Action::ToggleScope { mine_fresh: false }),
        [Effect::SaveScope(Scope::OnlyMe)]
    );
}

#[test]
fn a_late_counts_reply_is_ignored() {
    let (mut state, _) = load(cold());
    let effects = state.update(Action::ToggleScope { mine_fresh: false });
    let older = ticket_of(&effects, my_counts_ticket);
    let _ = state.update(Action::ToggleScope { mine_fresh: false });
    let _ = state.update(Action::ToggleScope { mine_fresh: false });
    let _ = state.update(Action::MyCountsLoaded {
        ticket: older,
        figures: figures(1),
    });
    assert_eq!(state.figures(), Figures::default(), "a stale reply landed");

    let (mut state, effects) = load(cold());
    let older = ticket_of(&effects, counts_ticket);
    let _ = state.update(Action::Load(Box::new(cold())));
    let _ = state.update(Action::CountsLoaded {
        ticket: older,
        figures: figures(1),
    });
    assert_eq!(state.figures(), Figures::default(), "a stale reply landed");
}

#[test]
fn the_color_map_draws_only_with_decks_and_cards() {
    for (summary, want) in [
        (summary(0, [1, 0, 0, 0, 0], 0), PlayStyle::Empty),
        (summary(2, [0; 5], 0), PlayStyle::Empty),
        (
            summary(2, [0; 5], 3),
            PlayStyle::Bars(ColorMapCounts {
                cards: [0; 5],
                colorless: 3,
            }),
        ),
        (
            HttpPlayStyleSummary {
                deck_count: 2,
                color_map: None,
            },
            PlayStyle::Empty,
        ),
    ] {
        let (state, _) = load(Cached {
            play_style: Some(summary.clone()),
            ..cold()
        });
        assert_eq!(state.play_style(), want, "{summary:?}");
    }
}

#[test]
fn the_decks_panel_lists_the_latest_three_in_the_servers_order() {
    let (mut state, effects) = load(cold());
    assert_eq!(state.decks(), Decks::Loading);
    let ticket = ticket_of(&effects, decks_ticket);
    let _ = state.update(Action::DecksLoaded {
        ticket,
        decks: vec![deck(4, "d"), deck(1, "a"), deck(3, "c"), deck(2, "b")],
    });
    assert_eq!(
        state.decks(),
        Decks::Loaded(DeckPanel {
            count: 4,
            latest: vec![deck(4, "d"), deck(1, "a"), deck(3, "c")],
        }),
        "the first three as listed, the count of them all"
    );

    let ticket = ticket_of(&effects, play_style_ticket);
    let _ = state.update(Action::PlayStyleLoaded {
        ticket,
        summary: summary(1, [0; 5], 0),
    });
    let Decks::Loaded(panel) = state.decks() else {
        panic!("loaded");
    };
    assert_eq!(panel.count, 4, "the summary counts only decks with cards");
}

#[test]
fn the_decks_panel_with_no_deck_offers_create_and_a_failure_says_so_unless_held() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, decks_ticket);
    let _ = state.update(Action::DecksLoaded {
        ticket,
        decks: Vec::new(),
    });
    assert_eq!(
        state.decks(),
        Decks::Loaded(DeckPanel {
            count: 0,
            latest: Vec::new(),
        })
    );

    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, decks_ticket);
    let effects = state.update(Action::DecksFailed { ticket });
    assert!(effects.is_empty(), "the client reports it: {effects:?}");
    assert_eq!(state.decks(), Decks::Failed);

    let (mut state, effects) = load(warm());
    assert_eq!(
        state.decks(),
        Decks::Loaded(DeckPanel {
            count: 2,
            latest: vec![deck(1, "Elves"), deck(2, "Goblins")],
        }),
        "held before the refetch lands"
    );
    let ticket = ticket_of(&effects, decks_ticket);
    let _ = state.update(Action::DecksFailed { ticket });
    assert!(
        matches!(state.decks(), Decks::Loaded(_)),
        "the held decks stay"
    );
}

#[test]
fn a_late_decks_reply_is_ignored() {
    let (mut state, effects) = load(cold());
    let older = ticket_of(&effects, decks_ticket);
    let _ = state.update(Action::Load(Box::new(cold())));
    let _ = state.update(Action::DecksLoaded {
        ticket: older,
        decks: vec![deck(1, "a")],
    });
    assert_eq!(state.decks(), Decks::Loading);
}

#[test]
fn a_failed_summary_says_so_unless_one_is_held() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, play_style_ticket);
    let effects = state.update(Action::PlayStyleFailed { ticket });
    assert!(effects.is_empty(), "the client reports it: {effects:?}");
    assert_eq!(state.play_style(), PlayStyle::Failed);

    let (mut state, effects) = load(warm());
    let ticket = ticket_of(&effects, play_style_ticket);
    let _ = state.update(Action::PlayStyleFailed { ticket });
    assert!(
        matches!(state.play_style(), PlayStyle::Bars(_)),
        "the held summary stays"
    );
}

#[test]
fn a_fetched_summary_replaces_the_held_one_and_clears_the_failure() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, play_style_ticket);
    let _ = state.update(Action::PlayStyleFailed { ticket });
    let _ = state.update(Action::PlayStyleLoaded {
        ticket,
        summary: summary(1, [0; 5], 0),
    });
    assert_eq!(state.play_style(), PlayStyle::Empty);
}

#[test]
fn a_late_summary_reply_is_ignored() {
    let (mut state, effects) = load(cold());
    let older = ticket_of(&effects, play_style_ticket);
    let _ = state.update(Action::Load(Box::new(cold())));
    let _ = state.update(Action::PlayStyleLoaded {
        ticket: older,
        summary: summary(1, [1; 5], 0),
    });
    assert_eq!(
        state.play_style(),
        PlayStyle::Loading,
        "a stale reply landed"
    );
    let _ = state.update(Action::PlayStyleFailed { ticket: older });
    assert_eq!(
        state.play_style(),
        PlayStyle::Loading,
        "a stale failure landed"
    );
}

#[test]
fn the_flavor_card_quotes_with_its_price_and_buy_links() {
    let (state, _) = load(warm());
    let Flavor::Quote(quote) = state.flavor() else {
        panic!("no quote: {:?}", state.flavor());
    };
    assert_eq!(quote.text, "One bone broken");
    assert_eq!(quote.name, "Llanowar Elves");
    assert_eq!(quote.price.as_deref(), Some("$0.08"));
    let links = state.buy_links();
    assert_eq!(
        links,
        [
            BuyLink {
                label: TCGPLAYER,
                url: "https://www.tcgplayer.com/product/91237".to_owned(),
            },
            BuyLink {
                label: CARD_KINGDOM,
                url: "https://www.cardkingdom.com/builder?c=1+Llanowar+Elves".to_owned(),
            },
        ]
    );
}

#[test]
fn an_unpriced_card_has_no_price_to_buy_at() {
    let mut held = flavor("Elves", Some("Hm"));
    held.card.scryfall_data.prices.usd = None;
    let (state, _) = load(Cached {
        flavor: Some(held),
        ..warm()
    });
    let Flavor::Quote(quote) = state.flavor() else {
        panic!("no quote");
    };
    assert_eq!(quote.price, None);
}

#[test]
fn a_card_without_flavor_text_leaves_the_panel_blank() {
    let (state, _) = load(Cached {
        flavor: Some(flavor("Elves", None)),
        ..warm()
    });
    assert_eq!(state.flavor(), Flavor::Blank);
    assert!(state.flavor_card().is_some(), "the card is still held");
}

#[test]
fn a_blank_flavor_text_still_quotes_as_in_the_app() {
    let (state, _) = load(Cached {
        flavor: Some(flavor("Elves", Some(""))),
        ..warm()
    });
    assert!(matches!(state.flavor(), Flavor::Quote(q) if q.text.is_empty()));
}

#[test]
fn an_expired_card_shows_while_the_next_is_fetched() {
    let (mut state, effects) = load(Cached {
        now: noon() + Duration::hours(1),
        ..warm()
    });
    let ticket = ticket_of(&effects, flavor_ticket);
    assert!(
        matches!(state.flavor(), Flavor::Quote(_)),
        "the old card stays"
    );
    let next = flavor("Fog", Some("Rain"));
    let _ = state.update(Action::FlavorLoaded {
        ticket,
        flavor: Box::new(next.clone()),
    });
    assert_eq!(state.flavor_card(), Some(&next.card));
}

#[test]
fn check_flavor_fetches_once_the_hour_turns_and_never_twice_at_once() {
    let (mut state, _) = load(warm());
    assert!(
        state.update(Action::CheckFlavor { now: noon() }).is_empty(),
        "the card is fresh"
    );
    let later = noon() + Duration::hours(1);
    let effects = state.update(Action::CheckFlavor { now: later });
    let ticket = ticket_of(&effects, flavor_ticket);
    assert!(
        state.update(Action::CheckFlavor { now: later }).is_empty(),
        "a fetch is out"
    );
    let _ = state.update(Action::FlavorFailed { ticket });
    assert!(
        state
            .update(Action::CheckFlavor { now: later })
            .iter()
            .any(|e| flavor_ticket(e).is_some()),
        "a failure lets the next check ask again"
    );
}

#[test]
fn a_cold_load_and_its_first_check_fetch_the_card_once() {
    let (mut state, _) = load(cold());
    let effects = state.update(Action::CheckFlavor { now: noon() });
    assert!(effects.is_empty(), "a second fetch: {effects:?}");
}

#[test]
fn a_failed_flavor_fetch_keeps_the_ghost() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, flavor_ticket);
    let effects = state.update(Action::FlavorFailed { ticket });
    assert!(effects.is_empty(), "a failure only logs: {effects:?}");
    assert_eq!(state.flavor(), Flavor::Loading);
}

#[test]
fn a_late_flavor_reply_is_ignored() {
    let (mut state, effects) = load(cold());
    let older = ticket_of(&effects, flavor_ticket);
    let _ = state.update(Action::Load(Box::new(cold())));
    let _ = state.update(Action::FlavorLoaded {
        ticket: older,
        flavor: Box::new(flavor("Old", Some("Stale"))),
    });
    assert_eq!(state.flavor(), Flavor::Loading, "a stale reply landed");
}

#[test]
fn a_flavor_card_expires_at_the_top_of_the_hour() {
    let held = flavor("Elves", None);
    assert_eq!(
        held.expires_at,
        Utc.with_ymd_and_hms(2026, 10, 30, 13, 0, 0).unwrap()
    );
    assert!(!held.is_expired(held.expires_at - Duration::seconds(1)));
    assert!(held.is_expired(held.expires_at));
}

#[test]
fn a_verified_user_is_greeted_once_per_launch() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, user_ticket);
    let effects = state.update(Action::UserLoaded {
        ticket,
        user: user(true),
        greeted: false,
    });
    assert_eq!(
        effects,
        [
            Effect::MarkGreeted,
            Effect::Toast(Toast::quick(ToastKind::Info, "Hello, scotty!"))
        ]
    );
    assert!(
        state
            .update(Action::UserLoaded {
                ticket,
                user: user(true),
                greeted: true,
            })
            .is_empty(),
        "already greeted"
    );
}

#[test]
fn an_unverified_user_is_nudged_every_visit() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, user_ticket);
    for greeted in [false, true] {
        assert_eq!(
            state.update(Action::UserLoaded {
                ticket,
                user: user(false),
                greeted,
            }),
            [Effect::Toast(Toast::quick(
                ToastKind::Warning,
                home::VERIFY_EMAIL
            ))],
            "greeted {greeted}"
        );
    }
}

#[test]
fn a_failed_user_refresh_greets_from_the_session_but_never_nudges() {
    let (mut state, effects) = load(cold());
    let ticket = ticket_of(&effects, user_ticket);
    let effects = state.update(Action::UserFailed {
        ticket,
        cached: Some(user(true)),
        greeted: false,
    });
    assert_eq!(effects.first(), Some(&Effect::MarkGreeted));
    for cached in [Some(user(false)), None] {
        assert!(
            state
                .update(Action::UserFailed {
                    ticket,
                    cached,
                    greeted: false,
                })
                .is_empty(),
            "an unverified or missing session user"
        );
    }
}

#[test]
fn a_late_user_reply_is_ignored() {
    let (mut state, effects) = load(cold());
    let older = ticket_of(&effects, user_ticket);
    let _ = state.update(Action::Load(Box::new(cold())));
    assert!(
        state
            .update(Action::UserLoaded {
                ticket: older,
                user: user(true),
                greeted: false,
            })
            .is_empty(),
        "a stale user landed"
    );
    assert!(
        state
            .update(Action::UserFailed {
                ticket: older,
                cached: Some(user(true)),
                greeted: false,
            })
            .is_empty(),
        "a stale failure landed"
    );
}

#[test]
fn every_button_navigates() {
    let mut state = State::new();
    for (action, to) in [
        (Action::Decks, Destination::Decks),
        (
            Action::OpenDeck(Uuid::from_u128(9)),
            Destination::Deck(Uuid::from_u128(9)),
        ),
        (Action::CreateDeck, Destination::CreateDeck),
        (Action::Account, Destination::Account),
        (Action::Config, Destination::Config),
        (Action::Activity, Destination::Activity),
        (Action::Playstyle, Destination::Playstyle),
    ] {
        assert_eq!(state.update(action), [Effect::Navigate(to)]);
    }
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [TITLE, LIVE, ACTIVITY, DECKS, ACCOUNT, CONFIG],
        ["Home", "Live", "Activity", "Decks", "Account", "Settings"]
    );
    assert_eq!(
        [ALL_DECKS, CREATE_DECK, DECKS_FAILED],
        ["All decks", "Create a deck", "Could not load your decks"]
    );
    assert_eq!(
        [COLORS, PLAYSTYLE, NO_PLAY_STYLE, PLAY_STYLE_FAILED],
        [
            "Colors",
            "Playstyle",
            "Build a deck to see your playstyle",
            "Could not load your playstyle"
        ]
    );
    assert_eq!(
        [
            FEATURED_FLAVOR,
            IMAGE,
            BUY,
            BUY_TITLE,
            BUY_BODY,
            CLOSE,
            TCGPLAYER,
            CARD_KINGDOM
        ],
        [
            "Featured flavor",
            "Image",
            "Buy",
            "Buy card",
            "Buy this card from a retailer.",
            "Close",
            "TCGplayer \u{2197}",
            "Card Kingdom \u{2197}"
        ]
    );
}
