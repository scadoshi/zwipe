use zwipe_core::domain::deck::{DeckOtherTag, Format, WarningAction};

use super::*;
use crate::test_support::{card, deck, deck_profile};
use crate::toast::{QUICK, ToastKind};

/// A state that has sent `Load` for `profile`'s deck, with the profile and deck tickets.
fn loading(profile: &DeckProfile) -> (State, Ticket, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load(profile.id));
    let [
        Effect::FetchProfile {
            ticket: profile_ticket,
            ..
        },
        Effect::FetchDeck {
            ticket: deck_ticket,
            ..
        },
        Effect::OfferHint(OneTimeHint::FirstDeck),
    ] = effects.as_slice()
    else {
        panic!("Load fetches and offers the hint: {effects:?}");
    };
    (state, *profile_ticket, *deck_ticket)
}

/// A state with `profile` and `built` loaded.
fn loaded(profile: &DeckProfile, built: Deck) -> State {
    let (mut state, profile_ticket, deck_ticket) = loading(profile);
    let _ = state.update(Action::ProfileLoaded {
        ticket: profile_ticket,
        profile: Box::new(profile.clone()),
    });
    let _ = state.update(Action::DeckLoaded {
        ticket: deck_ticket,
        deck: Box::new(built),
    });
    state
}

/// A deck named `name` holding one copy each of two cards.
fn two_cards(name: &str) -> (DeckProfile, Deck) {
    let profile = deck_profile(name);
    let built = deck(profile.clone(), vec![card("Sol Ring"), card("Opt")]);
    (profile, built)
}

/// The ticket of the one request `effects` sends.
fn ticket_of(effects: &[Effect]) -> Ticket {
    match effects {
        [
            Effect::SetQuantity { ticket, .. }
            | Effect::ClearCommander { ticket, .. }
            | Effect::RemoveCard { ticket, .. }
            | Effect::CloneDeck { ticket, .. }
            | Effect::ShareDeck { ticket, .. }
            | Effect::UnshareDeck { ticket, .. }
            | Effect::ClearSkips { ticket, .. }
            | Effect::DeleteDeck { ticket, .. },
        ] => *ticket,
        _ => panic!("not one request: {effects:?}"),
    }
}

fn info(text: &str) -> Effect {
    Effect::Toast(Toast::new(ToastKind::Info, text, QUICK))
}

#[test]
fn a_new_state_is_loading_with_no_deck() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.deck_id(), None);
    assert!(state.deck_loading());
    assert_eq!(state.sections(), []);
    assert!(!state.stats_loading());
    assert_eq!(state.currency(), PriceCurrency::Usd);
    assert_eq!(state.clone_name(), "");
    assert_eq!(state.card_count(), 0);
    assert_eq!(state.buy_body(), BUY_NONE);
    assert_eq!(state.share_body(), SHARE_UNSHARED);
}

#[test]
fn nothing_acts_before_load() {
    let mut state = State::new();
    for action in [
        Action::Edit,
        Action::Cards,
        Action::AddCards,
        Action::More(MoreItem::ImportCards),
        Action::ClearCommander,
        Action::Share,
        Action::Delete,
    ] {
        assert_eq!(state.update(action.clone()), [], "{action:?}");
    }
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::DeckList)]
    );
}

#[test]
fn load_fetches_both_and_offers_the_hint_once_per_visit() {
    let (profile, _) = two_cards("Elves");
    let (mut state, _, _) = loading(&profile);
    assert_eq!(state.deck_id(), Some(profile.id));
    let effects = state.update(Action::Load(profile.id));
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchProfile { .. }, Effect::FetchDeck { .. }]
        ),
        "the hint came again: {effects:?}"
    );
}

#[test]
fn the_profile_lands_and_seeds_the_currency_and_share_link() {
    let mut profile = deck_profile("Budget");
    profile.price_target_currency = Some(PriceCurrency::Eur);
    profile.share_token = Some(Uuid::nil());
    let (mut state, ticket, _) = loading(&profile);
    let effects = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile.clone()),
    });
    assert_eq!(effects, []);
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.profile(), Some(&profile));
    assert_eq!(state.currency(), PriceCurrency::Eur);
    assert_eq!(state.share_token(), Some(Uuid::nil()));
    assert_eq!(state.share_body(), SHARE_SHARED);
    assert_eq!(state.clone_name(), "Budget (clone)");
    assert!(state.deck_loading(), "the deck is still out");
}

#[test]
fn the_currency_seeds_once_per_visit_and_then_follows_the_pick() {
    let mut profile = deck_profile("Budget");
    profile.price_target_currency = Some(PriceCurrency::Tix);
    let mut state = loaded(&profile, deck(profile.clone(), Vec::new()));
    let _ = state.update(Action::PickCurrency(PriceCurrency::Usd));
    assert_eq!(state.currency(), PriceCurrency::Usd);

    let effects = state.update(Action::Load(profile.id));
    let Some(Effect::FetchProfile { ticket, .. }) = effects.first() else {
        panic!("Load fetches the profile: {effects:?}");
    };
    let _ = state.update(Action::ProfileLoaded {
        ticket: *ticket,
        profile: Box::new(profile),
    });
    assert_eq!(
        state.currency(),
        PriceCurrency::Usd,
        "the pick snapped back"
    );
}

#[test]
fn a_commander_is_fetched_with_the_profile_and_lands() {
    let commander = card("Krenko, Mob Boss");
    let mut profile = deck_profile("Goblins");
    profile.commander_id = Some(commander.scryfall_data.id);
    let (mut state, ticket, _) = loading(&profile);
    let effects = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile),
    });
    let [Effect::FetchCommander { ticket, card: id }] = effects.as_slice() else {
        panic!("the commander is fetched: {effects:?}");
    };
    assert_eq!(*id, commander.scryfall_data.id);
    let _ = state.update(Action::CommanderLoaded {
        ticket: *ticket,
        card: Box::new(commander.clone()),
    });
    assert_eq!(state.commander(), Some(&commander));
}

#[test]
fn a_failed_commander_leaves_the_profile_shown() {
    let mut profile = deck_profile("Goblins");
    profile.commander_id = Some(Uuid::new_v4());
    let (mut state, ticket, _) = loading(&profile);
    let effects = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile),
    });
    let [Effect::FetchCommander { ticket, .. }] = effects.as_slice() else {
        panic!("the commander is fetched: {effects:?}");
    };
    assert_eq!(state.update(Action::Failed { ticket: *ticket }), []);
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.commander(), None);
}

#[test]
fn a_failed_profile_reads_could_not_load() {
    let profile = deck_profile("Gone");
    let (mut state, ticket, _) = loading(&profile);
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.sections(), []);
}

#[test]
fn a_failed_deck_stops_loading_with_no_stats() {
    let (mut profile, _) = two_cards("Gone");
    profile.card_count = 2;
    let (mut state, profile_ticket, deck_ticket) = loading(&profile);
    let _ = state.update(Action::ProfileLoaded {
        ticket: profile_ticket,
        profile: Box::new(profile),
    });
    assert!(state.stats_loading());
    let _ = state.update(Action::Failed {
        ticket: deck_ticket,
    });
    assert!(!state.deck_loading());
    assert!(!state.stats_loading());
    assert_eq!(state.metrics(), None);
    assert_eq!(state.status(), Status::Loaded);
}

#[test]
fn a_late_reply_is_ignored() {
    let (profile, built) = two_cards("Elves");
    let (mut state, old_profile, old_deck) = loading(&profile);
    let _ = state.update(Action::Load(profile.id));
    let _ = state.update(Action::ProfileLoaded {
        ticket: old_profile,
        profile: Box::new(profile),
    });
    let _ = state.update(Action::DeckLoaded {
        ticket: old_deck,
        deck: Box::new(built),
    });
    assert_eq!(state.status(), Status::Loading);
    assert!(state.deck_loading());
    let _ = state.update(Action::Failed {
        ticket: old_profile,
    });
    assert_eq!(state.status(), Status::Loading, "a stale failure landed");
}

#[test]
fn a_late_commander_is_ignored() {
    let mut profile = deck_profile("Goblins");
    profile.commander_id = Some(Uuid::new_v4());
    let (mut state, ticket, _) = loading(&profile);
    let effects = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile.clone()),
    });
    let [Effect::FetchCommander { ticket: old, .. }] = effects.as_slice() else {
        panic!("the commander is fetched: {effects:?}");
    };
    let old = *old;
    let effects = state.update(Action::Load(profile.id));
    let Some(Effect::FetchProfile { ticket, .. }) = effects.first() else {
        panic!("Load fetches the profile: {effects:?}");
    };
    let _ = state.update(Action::ProfileLoaded {
        ticket: *ticket,
        profile: Box::new(profile),
    });
    let _ = state.update(Action::CommanderLoaded {
        ticket: old,
        card: Box::new(card("Stale")),
    });
    assert_eq!(state.commander(), None);
}

#[test]
fn a_reload_keeps_the_deck_shown_until_the_reply() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let _ = state.update(Action::Load(profile.id));
    assert_eq!(state.status(), Status::Loaded);
    assert!(!state.deck_loading());
    assert!(state.has_cards());
}

#[test]
fn the_deck_lands_with_metrics_and_stats_sections() {
    let (mut profile, built) = two_cards("Elves");
    profile.card_count = 2;
    let state = loaded(&profile, built);
    assert!(state.has_cards());
    assert_eq!(state.card_count(), 2);
    assert!(state.charted().is_some());
    assert_eq!(
        state.sections(),
        [
            Section::Budget,
            Section::Distributions,
            Section::Mana,
            Section::DrawOdds
        ],
        "several sections show at once"
    );
    assert!(!state.stats_loading());
    assert_eq!(state.buy_body(), BUY_BODY);
    assert!(state.currency_chips_shown());
}

#[test]
fn an_empty_deck_shows_no_stats_and_falls_back_to_the_profile_count() {
    let mut profile = deck_profile("Empty");
    profile.card_count = 3;
    let state = loaded(&profile, deck(profile.clone(), Vec::new()));
    assert_eq!(state.metrics(), None);
    assert!(!state.has_cards());
    assert_eq!(state.card_count(), 3);
    assert_eq!(state.sections(), []);
    assert!(!state.currency_chips_shown());
}

#[test]
fn a_land_target_shows_mana_and_hides_the_skeleton() {
    let mut profile = deck_profile("Lands");
    profile.land_target = Some(36);
    profile.card_count = 1;
    let (mut state, ticket, _) = loading(&profile);
    let _ = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile),
    });
    assert_eq!(state.sections(), [Section::Mana]);
    assert!(!state.stats_loading(), "Mana stands in for the skeleton");
    assert_eq!(state.lands_line(), "0 / 36");
}

#[test]
fn the_skeleton_waits_for_a_deck_that_could_have_stats() {
    let mut profile = deck_profile("Fresh");
    let (mut state, ticket, _) = loading(&profile);
    let _ = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile.clone()),
    });
    assert!(!state.stats_loading(), "an empty deck has no stats coming");

    profile.commander_id = Some(Uuid::new_v4());
    let (mut state, ticket, _) = loading(&profile);
    let _ = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile),
    });
    assert!(state.stats_loading());
}

#[test]
fn tags_and_warnings_show_with_their_badges() {
    let (mut profile, mut built) = two_cards("Tagged");
    profile.tags = vec!["tokens".to_owned()];
    profile.other_tags = vec![DeckOtherTag::Jank];
    built.warnings = vec![
        DeckWarning::new("deck has 2 cards, commander requires at least 100"),
        DeckWarning::with_card("Opt is banned", Uuid::new_v4()),
    ];
    let state = loaded(&profile, built);
    let sections = state.sections();
    assert!(sections.contains(&Section::Tags), "{sections:?}");
    assert_eq!(sections.last(), Some(&Section::Warnings));
    assert_eq!(state.badge(Section::Tags).as_deref(), Some("2"));
    assert_eq!(state.badge(Section::Warnings).as_deref(), Some("2"));
    assert_eq!(state.badge(Section::Budget), None);
    assert_eq!(state.warnings().len(), 2);
}

#[test]
fn the_budget_lines_follow_the_picked_currency() {
    let (mut profile, built) = two_cards("Budget");
    profile.price_target = Some(100.0);
    let mut state = loaded(&profile, built);
    let total = state.total_price_line();
    assert!(
        total.as_deref().is_some_and(|t| t.ends_with(" / $100.00")),
        "{total:?}"
    );
    let _ = state.update(Action::PickCurrency(PriceCurrency::Tix));
    let metrics = state.metrics().cloned();
    let average = state.average_price_line();
    let want = metrics.and_then(|m| m.avg_price_tix).map_or_else(
        || NO_PRICE.to_owned(),
        |v| PriceCurrency::Tix.format_amount(v),
    );
    assert_eq!(average, Some(want));
}

#[test]
fn the_average_lines_hide_without_metrics() {
    let profile = deck_profile("Empty");
    let state = loaded(&profile, deck(profile.clone(), Vec::new()));
    assert_eq!(state.average_price_line(), None);
    assert_eq!(state.average_pt_line(), None);
    assert_eq!(state.average_mana_value_line(), None);
    assert_eq!(state.lands_line(), "0");
    assert_eq!(state.total_price_line().as_deref(), Some("$0.00"));
}

#[test]
fn the_action_bar_and_more_items_go_where_the_app_goes() {
    let (profile, built) = two_cards("Elves");
    let id = profile.id;
    let mut state = loaded(&profile, built);
    let cases = [
        (Action::Back, Effect::Navigate(Destination::DeckList)),
        (Action::Edit, Effect::Navigate(Destination::EditDeck(id))),
        (Action::Cards, Effect::Navigate(Destination::DeckCards(id))),
        (
            Action::AddCards,
            Effect::Navigate(Destination::AddCards(id)),
        ),
        (
            Action::More(MoreItem::ImportCards),
            Effect::Navigate(Destination::ImportDeck(id)),
        ),
        (
            Action::More(MoreItem::ExportCards),
            Effect::Navigate(Destination::ExportDeck(id)),
        ),
        (
            Action::More(MoreItem::BuyDeck),
            Effect::OpenDialog(Dialog::Buy),
        ),
        (
            Action::More(MoreItem::CloneDeck),
            Effect::OpenDialog(Dialog::Clone),
        ),
        (
            Action::More(MoreItem::ShareDeck),
            Effect::OpenDialog(Dialog::Share),
        ),
        (
            Action::More(MoreItem::ClearSkips),
            Effect::OpenDialog(Dialog::ClearSkips),
        ),
        (
            Action::More(MoreItem::DeleteDeck),
            Effect::OpenDialog(Dialog::Delete),
        ),
    ];
    for (action, effect) in cases {
        assert_eq!(state.update(action.clone()), [effect], "{action:?}");
    }
}

#[test]
fn export_and_buy_wait_for_a_card() {
    let profile = deck_profile("Empty");
    let mut state = loaded(&profile, deck(profile.clone(), Vec::new()));
    for item in MoreItem::ALL {
        let off = matches!(item, MoreItem::ExportCards | MoreItem::BuyDeck);
        assert_eq!(state.is_enabled(item), !off, "{item:?}");
    }
    assert_eq!(state.update(Action::More(MoreItem::ExportCards)), []);
    assert_eq!(state.update(Action::More(MoreItem::BuyDeck)), []);
}

#[test]
fn buy_links_put_the_profiles_command_zone_first() {
    let (mut profile, built) = two_cards("Krenko");
    profile.commander_name = Some("Krenko, Mob Boss".to_owned());
    let state = loaded(&profile, built);
    let (tcg, ck) = state.buy_links().unwrap();
    assert_eq!(
        tcg,
        "https://www.tcgplayer.com/massentry?c=1+Krenko%2C+Mob+Boss||1+Sol+Ring||1+Opt"
    );
    assert_eq!(
        ck,
        "https://www.cardkingdom.com/builder?c=1+Krenko%2C+Mob+Boss%0A1+Sol+Ring%0A1+Opt"
    );
}

#[test]
fn each_warning_offers_the_apps_buttons() {
    let card = Uuid::new_v4();
    let short = DeckWarning::new("deck has 60 cards, commander requires at least 100");
    assert_eq!(
        fix_buttons(&short),
        [
            ("Add cards".to_owned(), Action::AddCards),
            ("Import".to_owned(), Action::More(MoreItem::ImportCards)),
        ]
    );
    let copies = DeckWarning::with_action("Sol Ring exceeds", card, WarningAction::FixQuantity(1));
    assert_eq!(
        fix_buttons(&copies),
        [(
            "Fix to 1".to_owned(),
            Action::SetQuantity { card, quantity: 1 }
        )]
    );
    let commander = DeckWarning::with_action(
        "Opt is not a commander",
        card,
        WarningAction::ClearCommander,
    );
    assert_eq!(
        fix_buttons(&commander),
        [("Clear".to_owned(), Action::ClearCommander)]
    );
    let banned = DeckWarning::with_card("Opt is banned", card);
    assert_eq!(
        fix_buttons(&banned),
        [("Remove".to_owned(), Action::RemoveCard { card })]
    );
    assert_eq!(fix_buttons(&DeckWarning::new("40 lands")), []);
}

#[test]
fn a_quantity_fix_toasts_and_reloads_the_deck() {
    let (profile, built) = two_cards("Elves");
    let card = Uuid::new_v4();
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::SetQuantity { card, quantity: 1 });
    assert_eq!(
        effects,
        [Effect::SetQuantity {
            ticket: ticket_of(&effects),
            deck_id: profile.id,
            card,
            quantity: 1
        }]
    );
    let effects = state.update(Action::QuantitySet {
        ticket: ticket_of(&effects),
    });
    assert!(
        matches!(
            effects.as_slice(),
            [toast, Effect::FetchDeck { deck_id, .. }]
                if *toast == info("Quantity set to 1") && *deck_id == profile.id
        ),
        "{effects:?}"
    );
    assert!(state.has_cards(), "the deck stays shown through the reload");
}

#[test]
fn clearing_the_commander_toasts_its_label_and_reloads_both() {
    let mut profile = deck_profile("Oath");
    profile.format = Some(Format::Oathbreaker);
    profile.commander_id = Some(Uuid::new_v4());
    let (mut state, ticket, _) = loading(&profile);
    let effects = state.update(Action::ProfileLoaded {
        ticket,
        profile: Box::new(profile),
    });
    let [Effect::FetchCommander { ticket, .. }] = effects.as_slice() else {
        panic!("the commander is fetched: {effects:?}");
    };
    let _ = state.update(Action::CommanderLoaded {
        ticket: *ticket,
        card: Box::new(card("Wrenn")),
    });
    let effects = state.update(Action::ClearCommander);
    let effects = state.update(Action::CommanderCleared {
        ticket: ticket_of(&effects),
    });
    assert!(
        matches!(
            effects.as_slice(),
            [toast, Effect::FetchProfile { .. }, Effect::FetchDeck { .. }]
                if *toast == info("Oathbreaker cleared")
        ),
        "{effects:?}"
    );
    assert_eq!(state.commander(), None);
}

#[test]
fn removing_a_card_toasts_and_reloads_the_deck() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::RemoveCard {
        card: Uuid::new_v4(),
    });
    let effects = state.update(Action::CardRemoved {
        ticket: ticket_of(&effects),
    });
    assert!(
        matches!(effects.as_slice(), [toast, Effect::FetchDeck { .. }] if *toast == info("Card removed")),
        "{effects:?}"
    );
}

#[test]
fn a_clone_toasts_the_typed_name_and_opens_the_copy() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    assert_eq!(
        state.update(Action::Clone {
            name: "   ".to_owned()
        }),
        []
    );
    let effects = state.update(Action::Clone {
        name: "  Elves 2 ".to_owned(),
    });
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        [Effect::CloneDeck {
            ticket,
            deck_id: profile.id,
            name: "Elves 2".to_owned()
        }]
    );
    let copy = Uuid::new_v4();
    assert_eq!(
        state.update(Action::Cloned {
            ticket,
            deck_id: copy
        }),
        [
            info("Cloned as \"Elves 2\""),
            Effect::Navigate(Destination::Deck(copy))
        ]
    );
}

#[test]
fn sharing_copies_the_new_link_and_stopping_disables_it() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    assert_eq!(state.update(Action::CopyLink), [], "no link to copy yet");

    let effects = state.update(Action::Share);
    let token = Uuid::new_v4();
    let copied = Effect::Copy {
        text: view::share_link(token),
        done: Toast::new(ToastKind::Info, "Share link copied", QUICK),
    };
    assert_eq!(
        state.update(Action::Shared {
            ticket: ticket_of(&effects),
            token
        }),
        std::slice::from_ref(&copied)
    );
    assert_eq!(state.share_token(), Some(token));
    assert_eq!(state.update(Action::CopyLink), [copied]);

    let effects = state.update(Action::StopShare);
    assert_eq!(
        state.update(Action::Unshared {
            ticket: ticket_of(&effects)
        }),
        [info("Link disabled")]
    );
    assert_eq!(state.share_token(), None);
}

#[test]
fn clearing_skips_toasts() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::ClearSkips);
    assert_eq!(
        state.update(Action::SkipsCleared {
            ticket: ticket_of(&effects)
        }),
        [info("Skips cleared")]
    );
}

#[test]
fn a_delete_replaces_the_deck_with_the_list() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::Delete);
    assert_eq!(
        state.update(Action::Deleted {
            ticket: ticket_of(&effects)
        }),
        [Effect::Replace(Destination::DeckList)]
    );
}

#[test]
fn a_failed_request_lands_nothing_and_leaves_the_deck() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::Share);
    let ticket = ticket_of(&effects);
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert!(state.mutations.is_empty(), "the request still waits");
    assert!(state.has_cards());
    assert_eq!(
        state.update(Action::Shared {
            ticket,
            token: Uuid::new_v4()
        }),
        [],
        "a failed request landed"
    );
    assert_eq!(state.share_token(), None);
}

#[test]
fn a_reply_to_a_request_before_load_is_ignored() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::Delete);
    let _ = state.update(Action::Load(Uuid::new_v4()));
    assert_eq!(
        state.update(Action::Deleted {
            ticket: ticket_of(&effects)
        }),
        []
    );
}

#[test]
fn a_reply_answers_only_its_own_request() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let effects = state.update(Action::ClearSkips);
    assert_eq!(
        state.update(Action::Deleted {
            ticket: ticket_of(&effects)
        }),
        [],
        "a delete reply landed on a clear"
    );
}

#[test]
fn requests_in_flight_together_each_land() {
    let (profile, built) = two_cards("Elves");
    let mut state = loaded(&profile, built);
    let first = ticket_of(&state.update(Action::SetQuantity {
        card: Uuid::new_v4(),
        quantity: 1,
    }));
    let second = ticket_of(&state.update(Action::SetQuantity {
        card: Uuid::new_v4(),
        quantity: 2,
    }));
    let skips = ticket_of(&state.update(Action::ClearSkips));
    assert_eq!(
        state.update(Action::QuantitySet { ticket: second }).first(),
        Some(&info("Quantity set to 2"))
    );
    assert_eq!(
        state.update(Action::QuantitySet { ticket: first }).first(),
        Some(&info("Quantity set to 1"))
    );
    assert_eq!(
        state.update(Action::SkipsCleared { ticket: skips }),
        [info("Skips cleared")]
    );
}

#[test]
fn can_clone_needs_more_than_spaces() {
    assert!(!can_clone(""));
    assert!(!can_clone("   "));
    assert!(can_clone(" a "));
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Deck");
    assert_eq!(LOAD_FAILED, "Could not load deck");
    assert_eq!([BACK, EDIT, CARDS, MORE], ["Back", "Edit", "Cards", "More"]);
    assert_eq!(MORE_TITLE, "More actions");
    assert_eq!(
        MoreItem::ALL.map(MoreItem::label),
        [
            "Import cards",
            "Export cards",
            "Buy deck",
            "Clone deck",
            "Share deck",
            "Clear skips",
            "Delete deck"
        ]
    );
    assert_eq!(
        MoreItem::ALL.map(MoreItem::is_danger),
        [false, false, false, false, false, true, true]
    );
    assert_eq!(
        Section::ALL.map(Section::title),
        [
            "Budget",
            "Tags",
            "Distributions",
            "Mana",
            "Draw odds",
            "Warnings"
        ]
    );
    assert_eq!(
        [
            Dialog::Buy,
            Dialog::Clone,
            Dialog::Share,
            Dialog::ClearSkips,
            Dialog::Delete
        ]
        .map(Dialog::title),
        [
            "Buy deck",
            "Clone deck",
            "Share deck",
            "Clear skips",
            "Delete deck"
        ]
    );
    assert_eq!(
        [TOTAL_PRICE, AVERAGE_CARD_PRICE, NO_PRICE],
        ["Total price", "Average card price", "N/A"]
    );
    assert_eq!(
        [AVERAGE_PT, LANDS, AVERAGE_MANA_VALUE],
        ["Average P/T", "Lands", "Average mana value"]
    );
    assert_eq!([CLOSE, CANCEL], ["Close", "Cancel"]);
    assert_eq!(BUY_BODY, "Buy this deck's cards from a retailer.");
    assert_eq!(BUY_NONE, "No buy links are available for this deck.");
    assert_eq!(
        [TCGPLAYER, CARD_KINGDOM],
        ["TCGplayer \u{2197}", "Card Kingdom \u{2197}"]
    );
    assert_eq!(
        CLONE_BODY,
        "Make an exact copy of your deck under a new name."
    );
    assert_eq!(
        [CLONE_PLACEHOLDER, CLONE_SAVE, CLONE_BUSY],
        ["New deck name", "Save", "Cloning..."]
    );
    assert_eq!(
        SHARE_SHARED,
        "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one."
    );
    assert_eq!(
        SHARE_UNSHARED,
        "Anyone with the link can view this deck. You can stop sharing at any time."
    );
    assert_eq!(
        [COPY_LINK, STOP_SHARE, CREATE_LINK],
        ["Copy link", "Stop share", "Create link"]
    );
    assert_eq!(
        CLEAR_SKIPS_BODY,
        "Skipped cards come back on their own after a week, and removed cards after a month. Clearing brings them all back now. This can't be undone."
    );
    assert_eq!([CLEAR, DELETE], ["Clear", "Delete"]);
    assert_eq!(DELETE_BODY, "Are you sure you want to delete this deck?");
}

#[test]
fn every_short_label_drops_the_deck_or_cards_word() {
    for item in MoreItem::ALL {
        let short = item.short_label();
        assert!(item.label().starts_with(short), "{item:?}: {short}");
        assert!(
            !short.ends_with(" deck") && !short.ends_with(" cards"),
            "{item:?}: {short}"
        );
    }
}
