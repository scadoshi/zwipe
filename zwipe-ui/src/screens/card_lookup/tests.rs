use zwipe_core::domain::site::APP_BASE;
use zwipe_core::http::contracts::card::HttpCardStats;

use super::*;
use crate::test_support::card;
use crate::toast::QUICK;

fn named(slug: &str) -> Open {
    Open::Named {
        slug: slug.to_owned(),
    }
}

fn printing(slug: &str, id: Uuid) -> Open {
    Open::Printing {
        slug: slug.to_owned(),
        id,
    }
}

fn load(open: Open) -> Action {
    Action::Load {
        open,
        send: false,
        signed_in: true,
    }
}

fn parsed(slug: &str) -> CardSlug {
    slug.parse().unwrap()
}

/// `effects` without the stats fetch a card shown asks for, for tests about the rest.
fn besides_stats(effects: Vec<Effect>) -> Vec<Effect> {
    effects
        .into_iter()
        .filter(|e| !matches!(e, Effect::FetchStats { .. }))
        .collect()
}

/// A screen opened at `/card/<slug>` with its fetch out, and that fetch's ticket.
fn by_name(slug: &str, send: bool, signed_in: bool) -> (State, Ticket, Vec<Effect>) {
    let mut state = State::new();
    let effects = state.update(Action::Load {
        open: named(slug),
        send,
        signed_in,
    });
    let Some(Effect::FetchNamed { ticket, .. }) = effects.last() else {
        panic!("expected a name fetch: {effects:?}");
    };
    (state, *ticket, effects)
}

/// A screen opened at `/card/<slug>/<id>` with its fetch out, and that fetch's ticket.
fn by_printing(slug: &str, id: Uuid) -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(load(printing(slug, id)));
    let Some(Effect::FetchCard { ticket, .. }) = effects.last() else {
        panic!("expected a printing fetch: {effects:?}");
    };
    (state, *ticket)
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(NOT_FOUND, "No card by that name");
    assert_eq!(LINK_COPIED, "Share link copied");
    assert_eq!(SCREEN_TITLE, "Card");
    assert_eq!(
        several_title("Rampant Growth"),
        "Cards named Rampant Growth"
    );
}

#[test]
fn the_screen_is_titled_by_its_card_or_the_name_its_cards_share() {
    let (mut state, ticket, _) = by_name("gather-the-townsfolk", false, true);
    assert_eq!(state.screen_title(), SCREEN_TITLE, "while it loads");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![card("Gather the Townsfolk"), card("Gather, the Townsfolk")],
    });
    assert_eq!(state.screen_title(), "Cards named Gather the Townsfolk");

    let (mut state, ticket, _) = by_name("bind", false, true);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![card("Bind")],
    });
    assert_eq!(state.screen_title(), "Bind");

    let (mut state, ticket, _) = by_name("no-such-card", false, true);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![],
    });
    assert_eq!(state.screen_title(), SCREEN_TITLE);
}

#[test]
fn a_name_fetches_every_card_that_has_it() {
    let mut state = State::new();
    assert_eq!(state.status(), Status::Loading);
    let effects = state.update(load(named("bind")));
    assert!(
        matches!(effects.as_slice(), [Effect::FetchNamed { slug, .. }] if slug.as_str() == "bind"),
        "{effects:?}"
    );
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn one_card_shows_it_several_list_them_none_says_so() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    let bind = card("Bind");
    assert_eq!(
        besides_stats(state.update(Action::Named {
            ticket,
            cards: vec![bind.clone()]
        })),
        vec![]
    );
    assert_eq!(state.status(), Status::One);
    assert_eq!(state.card(), Some(&bind));
    assert_eq!(state.title(), None);

    let (mut state, ticket, _) = by_name("rampant-growth", false, true);
    let cards = vec![card("Rampant Growth"), card("Rampant, Growth")];
    let _ = state.update(Action::Named {
        ticket,
        cards: cards.clone(),
    });
    assert_eq!(state.status(), Status::Several);
    assert_eq!(state.cards(), cards.as_slice());
    assert_eq!(state.title().as_deref(), Some("Cards named Rampant Growth"));
    assert_eq!(state.card(), None);

    let (mut state, ticket, _) = by_name("no-such-card", false, true);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![],
    });
    assert_eq!(state.status(), Status::NotFound);
    assert_eq!(state.message(), Some(NOT_FOUND));
}

/// `card` with Scryfall layout `layout`.
fn laid_out(name: &str, layout: &str) -> Card {
    let mut card = card(name);
    card.scryfall_data.layout = layout.to_owned();
    card
}

#[test]
fn only_playable_cards_make_a_list() {
    let (mut state, ticket, _) = by_name("sol-ring", false, true);
    let ring = laid_out("Sol Ring", "normal");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![ring.clone(), laid_out("Sol Ring // Sol Ring", "art_series")],
    });
    assert_eq!(
        state.status(),
        Status::One,
        "an art card's front face is no rival"
    );
    assert_eq!(state.card(), Some(&ring));

    let (mut state, ticket, _) = by_name("rampant-growth", false, true);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![
            laid_out("Rampant Growth", "normal"),
            laid_out("Rampant, Growth", "normal"),
            laid_out("Rampant Growth", "token"),
        ],
    });
    assert_eq!(state.status(), Status::Several);
    assert_eq!(state.cards().len(), 2, "the list is the playable cards");

    let (mut state, ticket, _) = by_name("goblin", false, true);
    let tokens = vec![laid_out("Goblin", "token"), laid_out("Goblin", "token")];
    let _ = state.update(Action::Named {
        ticket,
        cards: tokens.clone(),
    });
    assert_eq!(
        state.status(),
        Status::Several,
        "no playable card: every match lists"
    );
    assert_eq!(state.cards(), tokens.as_slice());
}

#[test]
fn a_full_name_match_wins_over_front_faces() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    let bind = laid_out("Bind", "normal");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![bind.clone(), laid_out("Bind // Liberate", "split")],
    });
    assert_eq!(state.status(), Status::One, "Bind is Bind");
    assert_eq!(state.card(), Some(&bind));

    let (mut state, ticket, _) = by_name("delver-of-secrets", false, true);
    let delver = laid_out("Delver of Secrets // Insectile Aberration", "transform");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![delver.clone()],
    });
    assert_eq!(
        state.card(),
        Some(&delver),
        "a front face still opens its card"
    );

    let (mut state, ticket, _) = by_name("shared-front", false, true);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![
            laid_out("Shared Front // One Back", "modal_dfc"),
            laid_out("Shared Front // Other Back", "modal_dfc"),
        ],
    });
    assert_eq!(
        state.status(),
        Status::Several,
        "two fronts and no full name: both list"
    );
}

#[test]
fn a_route_that_is_not_a_slug_finds_nothing_without_asking() {
    let mut state = State::new();
    for slug in ["Sol Ring", "", "sol--ring"] {
        assert_eq!(state.update(load(named(slug))), vec![], "{slug:?}");
        assert_eq!(state.status(), Status::NotFound, "{slug:?}");
    }
}

#[test]
fn picking_one_of_several_opens_its_printing() {
    let (mut state, ticket, _) = by_name("rampant-growth", false, true);
    let comma = card("Rampant, Growth");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![card("Rampant Growth"), comma.clone()],
    });
    let id = comma.scryfall_data.id;
    assert_eq!(
        state.update(Action::Pick(id)),
        vec![Effect::Navigate(Destination::Printing {
            slug: "rampant-growth".to_owned(),
            id,
            send: false,
        })]
    );
    assert_eq!(
        state.update(Action::Pick(Uuid::nil())),
        vec![],
        "not listed"
    );
}

#[test]
fn a_printing_loads_by_its_id_and_fixes_a_stale_slug() {
    let fire = card("Fire // Ice");
    let id = fire.scryfall_data.id;
    let (mut state, ticket) = by_printing("fire-ice", id);
    assert_eq!(
        besides_stats(state.update(Action::Printing {
            ticket,
            card: Box::new(fire.clone())
        })),
        vec![],
        "the slug is already right"
    );
    assert_eq!(state.card(), Some(&fire));

    let (mut state, ticket) = by_printing("old-name", id);
    assert_eq!(
        besides_stats(state.update(Action::Printing {
            ticket,
            card: Box::new(fire)
        })),
        vec![Effect::Replace(Destination::Printing {
            slug: "fire-ice".to_owned(),
            id,
            send: false,
        })]
    );
}

#[test]
fn a_missing_printing_finds_nothing() {
    let (mut state, ticket) = by_printing("fire-ice", Uuid::nil());
    let _ = state.update(Action::Missing { ticket });
    assert_eq!(state.status(), Status::NotFound);
}

#[test]
fn a_failure_offers_retry_which_asks_again() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    let _ = state.update(Action::Failed { ticket });
    assert_eq!(state.status(), Status::Failed);
    let effects = state.update(Action::Retry);
    let Some(Effect::FetchNamed {
        ticket: again,
        slug,
    }) = effects.last()
    else {
        panic!("expected a name fetch: {effects:?}");
    };
    assert_ne!(*again, ticket);
    assert_eq!(slug.as_str(), "bind");
    assert_eq!(state.status(), Status::Loading);

    let (mut state, ticket) = by_printing("bind", Uuid::nil());
    let _ = state.update(Action::Failed { ticket });
    assert!(
        matches!(state.update(Action::Retry).as_slice(), [Effect::FetchCard { id, .. }] if id.is_nil())
    );
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, old, _) = by_name("bind", false, true);
    let _ = state.update(Action::Failed { ticket: old });
    let _ = state.update(Action::Retry);
    let _ = state.update(Action::Named {
        ticket: old,
        cards: vec![card("Bind")],
    });
    assert_eq!(state.status(), Status::Loading, "the retry's reply decides");
    let _ = state.update(Action::Failed { ticket: old });
    assert_eq!(state.status(), Status::Loading);

    let (mut state, old) = by_printing("bind", Uuid::nil());
    let _ = state.update(Action::Failed { ticket: old });
    let _ = state.update(Action::Retry);
    let _ = state.update(Action::Printing {
        ticket: old,
        card: Box::new(card("Bind")),
    });
    let _ = state.update(Action::Missing { ticket: old });
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn send_to_signed_out_goes_to_login_and_back_to_the_card() {
    let (mut state, ticket, _) = by_name("bind", false, false);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![card("Bind")],
    });
    assert_eq!(
        state.update(Action::SendTo { signed_in: false }),
        vec![
            Effect::RememberReturn(Destination::Named {
                slug: parsed("bind"),
                send: true,
            }),
            Effect::Navigate(Destination::Login),
        ]
    );

    let liberate = card("Bind // Liberate");
    let id = liberate.scryfall_data.id;
    let (mut state, ticket) = by_printing("bind-liberate", id);
    let _ = state.update(Action::Printing {
        ticket,
        card: Box::new(liberate),
    });
    assert_eq!(
        state.update(Action::SendTo { signed_in: false }),
        vec![
            Effect::RememberReturn(Destination::Printing {
                slug: "bind-liberate".to_owned(),
                id,
                send: true,
            }),
            Effect::Navigate(Destination::Login),
        ]
    );
}

#[test]
fn send_to_signed_in_opens_the_sheet() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    let bind = card("Bind");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![bind.clone()],
    });
    assert_eq!(
        state.update(Action::SendTo { signed_in: true }),
        vec![Effect::OpenSendTo(Box::new(bind))]
    );
}

#[test]
fn an_asked_for_send_to_opens_once_and_leaves_the_route() {
    let (mut state, ticket, effects) = by_name("bind", true, true);
    assert_eq!(
        effects.first(),
        Some(&Effect::Replace(Destination::Named {
            slug: parsed("bind"),
            send: false,
        })),
        "the ask leaves the route at once: {effects:?}"
    );
    let bind = card("Bind");
    assert_eq!(
        besides_stats(state.update(Action::Named {
            ticket,
            cards: vec![bind.clone()]
        })),
        vec![Effect::OpenSendTo(Box::new(bind))]
    );
    let _ = state.update(Action::Retry);
    let Some(Effect::FetchNamed { ticket, .. }) = state.update(Action::Retry).pop() else {
        panic!("expected a name fetch");
    };
    assert_eq!(
        besides_stats(state.update(Action::Named {
            ticket,
            cards: vec![card("Bind")]
        })),
        vec![],
        "never twice"
    );
}

#[test]
fn an_asked_for_send_to_waits_on_a_session_and_a_single_card() {
    let (mut state, ticket, effects) = by_name("bind", true, false);
    assert!(
        matches!(effects.first(), Some(Effect::Replace(_))),
        "{effects:?}"
    );
    assert_eq!(
        besides_stats(state.update(Action::Named {
            ticket,
            cards: vec![card("Bind")]
        })),
        vec![],
        "no session"
    );

    let (mut state, ticket, _) = by_name("rampant-growth", true, true);
    assert_eq!(
        state.update(Action::Named {
            ticket,
            cards: vec![card("Rampant Growth"), card("Rampant, Growth")]
        }),
        vec![],
        "several: no one card to send"
    );
}

#[test]
fn an_asked_for_send_to_on_a_list_goes_with_the_picked_card() {
    let (mut state, ticket, _) = by_name("rampant-growth", true, true);
    let comma = card("Rampant, Growth");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![card("Rampant Growth"), comma.clone()],
    });
    let id = comma.scryfall_data.id;
    assert_eq!(
        state.update(Action::Pick(id)),
        vec![Effect::Navigate(Destination::Printing {
            slug: "rampant-growth".to_owned(),
            id,
            send: true,
        })],
        "the printing opens with the sheet"
    );
    assert!(
        matches!(
            state.update(Action::Pick(id)).as_slice(),
            [Effect::Navigate(Destination::Printing { send: false, .. })]
        ),
        "the ask goes with one pick only"
    );
}

#[test]
fn share_copies_the_name_s_link_or_the_printing_s() {
    let (mut state, ticket, _) = by_name("delver-of-secrets", false, true);
    let delver = card("Delver of Secrets // Insectile Aberration");
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![delver.clone()],
    });
    assert_eq!(
        state.update(Action::Share),
        vec![Effect::Copy {
            text: format!("{APP_BASE}/card/delver-of-secrets-insectile-aberration"),
            done: Toast::new(ToastKind::Info, LINK_COPIED, QUICK),
        }],
        "the full name's link, not the alias it came in on"
    );

    let id = delver.scryfall_data.id;
    let (mut state, ticket) = by_printing("whatever", id);
    let _ = state.update(Action::Printing {
        ticket,
        card: Box::new(delver),
    });
    assert!(
        matches!(state.update(Action::Share).as_slice(), [Effect::Copy { text, .. }] if *text == format!("{APP_BASE}/card/delver-of-secrets-insectile-aberration/{id}"))
    );

    let blank = card("_____");
    let id = blank.scryfall_data.id;
    let (mut state, ticket) = by_printing("card", id);
    let _ = state.update(Action::Printing {
        ticket,
        card: Box::new(blank),
    });
    assert!(
        matches!(state.update(Action::Share).as_slice(), [Effect::Copy { text, .. }] if *text == format!("{APP_BASE}/card/card/{id}"))
    );
}

#[test]
fn send_to_and_share_need_a_card_on_screen() {
    let (mut state, _, _) = by_name("bind", false, true);
    assert_eq!(state.update(Action::SendTo { signed_in: true }), vec![]);
    assert_eq!(state.update(Action::SendTo { signed_in: false }), vec![]);
    assert_eq!(state.update(Action::Share), vec![]);
}

#[test]
fn the_send_to_ask_reads_back_from_its_query_value_and_nothing_else_does() {
    assert_eq!(Ask::SendTo.to_string().parse(), Ok(Ask::SendTo));
    assert_eq!(Ask::SendTo.to_string(), "send-to");
    for other in ["", "send", "Send-To", "share"] {
        assert!(other.parse::<Ask>().is_err(), "{other:?}");
    }
}

#[test]
fn the_ask_leaving_the_route_reloads_nothing_and_keeps_send_to_asked() {
    let (mut state, ticket, _) = by_name("bind", true, true);
    // The client hears its own Replace as the route changing: the same card, no ask.
    assert_eq!(
        state.update(Action::Load {
            open: named("bind"),
            send: false,
            signed_in: true,
        }),
        vec![],
        "no second fetch"
    );
    let bind = card("Bind");
    assert_eq!(
        besides_stats(state.update(Action::Named {
            ticket,
            cards: vec![bind.clone()]
        })),
        vec![Effect::OpenSendTo(Box::new(bind))],
        "the first fetch still answers and the ask still opens Send to"
    );
}

#[test]
fn another_card_in_place_loads_it() {
    let (mut state, _, _) = by_name("bind", false, true);
    let effects = state.update(load(named("sol-ring")));
    assert!(
        matches!(effects.last(), Some(Effect::FetchNamed { slug, .. }) if slug.as_str() == "sol-ring"),
        "{effects:?}"
    );
}

/// The stats fetch among `effects`, with its ticket and oracle id.
fn stats_fetch(effects: &[Effect]) -> Option<(Ticket, Uuid)> {
    effects.iter().find_map(|e| {
        if let Effect::FetchStats { ticket, oracle_id } = e {
            Some((*ticket, *oracle_id))
        } else {
            None
        }
    })
}

#[test]
fn one_card_shown_asks_for_its_stats_by_oracle_id() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    assert_eq!(state.stats(), CardStats::Loading);
    let bind = card("Bind");
    let oracle = bind.scryfall_data.oracle_id.unwrap();
    let effects = state.update(Action::Named {
        ticket,
        cards: vec![bind],
    });
    let (stats_ticket, asked) = stats_fetch(&effects).expect("a stats fetch");
    assert_eq!(asked, oracle);
    let counts = HttpCardStats {
        in_decks: Some(4),
        ..HttpCardStats::default()
    };
    let _ = state.update(Action::Stats {
        ticket: stats_ticket,
        stats: counts,
    });
    assert_eq!(state.stats(), CardStats::Loaded(counts));
}

#[test]
fn a_printing_shown_asks_for_its_stats_too() {
    let id = Uuid::new_v4();
    let (mut state, ticket) = by_printing("bind", id);
    let mut bind = card("Bind");
    bind.scryfall_data.id = id;
    let effects = state.update(Action::Printing {
        ticket,
        card: Box::new(bind),
    });
    assert!(stats_fetch(&effects).is_some(), "{effects:?}");
}

#[test]
fn several_cards_ask_for_no_stats() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    let effects = state.update(Action::Named {
        ticket,
        cards: vec![card("Bind"), card("Bind")],
    });
    assert_eq!(state.status(), Status::Several);
    assert!(stats_fetch(&effects).is_none(), "{effects:?}");
}

#[test]
fn a_failed_stats_fetch_shows_failed_and_retry_asks_again_with_a_new_ticket() {
    let (mut state, ticket, _) = by_name("bind", false, true);
    let effects = state.update(Action::Named {
        ticket,
        cards: vec![card("Bind")],
    });
    let (first, _) = stats_fetch(&effects).unwrap();
    let _ = state.update(Action::StatsFailed { ticket: first });
    assert_eq!(state.stats(), CardStats::Failed);
    let effects = state.update(Action::RetryStats);
    let (second, _) = stats_fetch(&effects).expect("retry fetches");
    assert_ne!(second, first);
    assert_eq!(state.stats(), CardStats::Loading);
    let _ = state.update(Action::Stats {
        ticket: first,
        stats: HttpCardStats::default(),
    });
    assert_eq!(
        state.stats(),
        CardStats::Loading,
        "a stale answer is dropped"
    );
}

#[test]
fn retrying_stats_with_no_card_or_no_failure_does_nothing() {
    let mut state = State::new();
    assert_eq!(state.update(Action::RetryStats), vec![]);
    let (mut state, ticket, _) = by_name("bind", false, true);
    let effects = state.update(Action::Named {
        ticket,
        cards: vec![card("Bind")],
    });
    assert!(stats_fetch(&effects).is_some());
    assert_eq!(state.update(Action::RetryStats), vec![], "still loading");
}

#[test]
fn create_account_goes_to_register_and_back_to_the_card() {
    let (mut state, ticket, _) = by_name("bind", false, false);
    let _ = state.update(Action::Named {
        ticket,
        cards: vec![card("Bind")],
    });
    assert_eq!(
        state.update(Action::CreateAccount),
        vec![
            Effect::RememberReturn(Destination::Named {
                slug: parsed("bind"),
                send: false,
            }),
            Effect::Navigate(Destination::Register),
        ]
    );
}

#[test]
fn create_account_before_the_card_shows_still_goes_to_register() {
    let (mut state, _, _) = by_name("bind", false, false);
    assert_eq!(
        state.update(Action::CreateAccount),
        vec![
            Effect::RememberReturn(Destination::Named {
                slug: parsed("bind"),
                send: false,
            }),
            Effect::Navigate(Destination::Register),
        ]
    );
}
