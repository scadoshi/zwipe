use crossterm::event::KeyCode;
use ratatui::buffer::Cell;
use zwipe_core::domain::deck::{DeckWarning, Format, WarningAction};

use super::layout::Pane;
use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::deck_form;
use crate::test_support::{card, deck, deck_profile, render, screen_text, user};

fn view(harness: &Harness) -> &DeckView {
    match harness.screen() {
        Screen::Deck(view) => view,
        other => panic!("not on the deck screen: {other:?}"),
    }
}

fn view_mut(harness: &mut Harness) -> &mut DeckView {
    match harness.screen_mut() {
        Screen::Deck(view) => view,
        other => panic!("not on the deck screen: {other:?}"),
    }
}

/// Every cell's symbol of the app drawn at `width` by `height`, row after row.
fn text_at(harness: &Harness, width: u16, height: u16) -> String {
    render(&harness.app, width, height)
        .backend()
        .buffer()
        .content()
        .iter()
        .map(Cell::symbol)
        .collect()
}

/// The sizes the deck screen lays out for: wide, medium and narrow.
const SIZES: [(u16, u16); 3] = [(200, 50), (140, 40), (80, 24)];

/// A deck of two elves and a forest, a land target of 36, over the deck list.
fn elves() -> Deck {
    let creature = |name: &str, cost: &str, cmc: f64| {
        let mut c = card(name);
        c.scryfall_data.mana_cost = Some(cost.to_owned());
        c.scryfall_data.cmc = Some(cmc);
        c.scryfall_data.type_line = Some("Creature — Elf".to_owned());
        c
    };
    let mut forest = card("Forest");
    forest.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
    let mut profile = deck_profile("Elves");
    profile.land_target = Some(36);
    deck(
        profile,
        vec![
            creature("A", "{G}", 1.0),
            creature("B", "{1}{G}", 2.0),
            forest,
        ],
    )
}

/// The deck screen over the list with `deck` loaded and nothing queued.
fn loaded(deck: Deck) -> Harness {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    harness.app.shared.user = Some(user(true));
    let profile = deck.deck_profile.clone();
    harness.open(|cx| open(cx, profile));
    harness.app.shared.hint = None;
    harness.apply(Update::Loaded(Box::new(deck)));
    harness.clear();
    harness
}

#[test]
fn opening_loads_the_deck_and_the_tag_catalog_and_shows_the_first_deck_hint() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    let profile = deck_profile("Pile");
    let id = profile.id;
    harness.open(|cx| open(cx, profile));
    assert_eq!(
        harness.pending(),
        [
            screens::Request::Deck(Request::Load(id)),
            screens::Request::Deck(Request::LoadOracleTags),
            screens::Request::MarkHintShown(HINT_KEY.to_owned()),
        ]
    );
    assert!(screen_text(&harness.app).contains("Welcome to your deck"));
}

#[test]
fn s_on_a_loaded_deck_starts_swiping_and_fetches_the_first_page() {
    let mut profile = deck_profile("Pile");
    profile.commander_id = Some(uuid::Uuid::new_v4());
    profile.format = Some(zwipe_core::domain::deck::Format::Commander);
    let mut harness = Harness::on(Screen::Deck(Box::new(DeckView::opening(profile.clone()))));
    harness.press(KeyCode::Char('s'));
    assert!(
        matches!(harness.screen(), Screen::Deck(_)),
        "nothing to swipe before the deck loads"
    );

    harness.apply(Update::Loaded(Box::new(deck(profile.clone(), Vec::new()))));
    harness.press(KeyCode::Char('s'));
    assert!(harness.swipe_view().loading_cards);
    assert!(matches!(
        harness.pending(),
        [screens::Request::Swipe(screens::swipe::Request::Search { deck_id, .. })] if *deck_id == profile.id
    ));
}

#[test]
fn tab_walks_the_panes_in_the_apps_order_and_wraps() {
    let mut harness = loaded(elves());
    assert_eq!(view(&harness).focused(), Pane::Profile);
    assert_eq!(
        view(&harness).panes(),
        [
            Pane::Profile,
            Pane::Section(Section::Budget),
            Pane::Section(Section::Distributions),
            Pane::Section(Section::Mana),
            Pane::Section(Section::DrawOdds),
        ]
    );
    for want in [
        Section::Budget,
        Section::Distributions,
        Section::Mana,
        Section::DrawOdds,
    ] {
        harness.press(KeyCode::Tab);
        assert_eq!(view(&harness).focused(), Pane::Section(want));
    }
    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).focused(), Pane::Profile, "Tab wraps");
    harness.press(KeyCode::BackTab);
    assert_eq!(
        view(&harness).focused(),
        Pane::Section(Section::DrawOdds),
        "Shift+Tab wraps back"
    );
}

#[test]
fn budget_switches_currency_and_draw_odds_steps_the_turn() {
    let mut harness = loaded(elves());
    harness.press(KeyCode::Char('l'));
    assert_eq!(
        view(&harness).currency,
        PriceCurrency::Usd,
        "only the focused Budget takes the currency keys"
    );
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Char('l'));
    assert_eq!(view(&harness).currency, PriceCurrency::Eur);
    harness.press(KeyCode::Char('h'));
    harness.press(KeyCode::Char('h'));
    assert_eq!(view(&harness).currency, PriceCurrency::Tix);
    harness.press(KeyCode::Char('+'));
    assert_eq!(view(&harness).turn, 0, "the turn keys wait for Draw odds");
    view_mut(&mut harness).focus_on(Pane::Section(Section::DrawOdds));
    harness.press(KeyCode::Char('-'));
    for _ in 0..25 {
        harness.press(KeyCode::Char('+'));
    }
    assert_eq!(view(&harness).turn, MAX_TURN);
    harness.press(KeyCode::Char('h'));
    assert_eq!(view(&harness).turn, MAX_TURN.saturating_sub(1));
    assert!(screen_text(&harness.app).contains("Odds of ≥1 by turn 19"));
}

#[test]
fn the_budget_reads_the_total_and_target_and_the_mana_reads_the_lands() {
    let mut harness = loaded(elves());
    view_mut(&mut harness).focus_on(Pane::Section(Section::Budget));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Total price"));
    assert!(screen.contains("USD"));
    assert!(screen.contains("Average card price"));
    view_mut(&mut harness).focus_on(Pane::Section(Section::Mana));
    let screen = screen_text(&harness.app);
    for expected in ["1 / 36", "Average mana value", "Mana curve", "Colors"] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn roles_expand_into_their_tags_and_a_tag_into_its_definition() {
    let mut ramp = card("Llanowar Elves");
    ramp.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    ramp.scryfall_data.cmc = Some(1.0);
    ramp.card_profile.card_roles = vec!["ramp".to_owned()];
    ramp.card_profile
        .oracle_tags_by_role
        .insert("ramp".to_owned(), vec!["mana-dork".to_owned()]);
    let built = deck(deck_profile("Ramp"), vec![ramp]);
    let mut harness = loaded(built);
    view_mut(&mut harness).focus_on(Pane::Section(Section::Distributions));
    let Some(Item::Role(role)) = view(&harness).focused_item() else {
        panic!("the first role is the first item");
    };
    harness.press(KeyCode::Char(' '));
    assert_eq!(view(&harness).role_open.as_deref(), Some(role.as_str()));
    harness.press(KeyCode::Char('j'));
    let Some(Item::Tag(tag)) = view(&harness).focused_item() else {
        panic!("the role's tags follow it");
    };
    harness.press(KeyCode::Char(' '));
    assert_eq!(view(&harness).tag_open.as_deref(), Some(tag.as_str()));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("No description yet"));
    assert!(screen.contains("Examples"));
}

#[test]
fn a_role_with_no_tags_is_a_grey_chip_the_focus_skips() {
    let mut ramp = card("Llanowar Elves");
    ramp.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    ramp.scryfall_data.cmc = Some(1.0);
    ramp.card_profile.card_roles = vec!["ramp".to_owned(), "draw".to_owned()];
    ramp.card_profile
        .oracle_tags_by_role
        .insert("ramp".to_owned(), vec!["mana-dork".to_owned()]);
    let built = deck(deck_profile("Ramp"), vec![ramp]);
    let mut harness = loaded(built);
    view_mut(&mut harness).focus_on(Pane::Section(Section::Distributions));
    let metrics = view(&harness).metrics.clone().unwrap();
    let bars = metrics.card_role_bars().unwrap();
    let untagged: Vec<&str> = bars
        .iter()
        .map(|(label, _, _)| *label)
        .filter(|label| !metrics.card_role_tags.contains_key(*label))
        .collect();
    assert_eq!(untagged.len(), 1, "one role has no tags: {bars:?}");
    let roles = view(&harness)
        .items()
        .into_iter()
        .filter(|t| matches!(t, Item::Role(_)))
        .count();
    assert_eq!(roles, 1, "the focus only rests on a role with tags");
    let chip = format!(" {} ", untagged[0].to_uppercase());
    assert!(
        screen_text(&harness.app).contains(&chip),
        "missing {chip:?}"
    );
}

#[test]
fn warnings_offer_their_fixes_and_run_them() {
    let mut built = elves();
    let sol_ring = built.entries[0].card.scryfall_data.id;
    built.warnings = vec![
        DeckWarning::new("deck has 3 cards, commander requires at least 100"),
        DeckWarning::with_action(
            "A exceeds copy limit (2/1)",
            sol_ring,
            WarningAction::FixQuantity(1),
        ),
        DeckWarning::with_card("B is banned in commander", sol_ring),
        DeckWarning::with_action(
            "A is not a valid commander for commander",
            sol_ring,
            WarningAction::ClearCommander,
        ),
    ];
    let mut harness = loaded(built);
    view_mut(&mut harness).focus_on(Pane::Section(Section::Warnings));
    let screen = screen_text(&harness.app);
    for expected in [
        "Warnings 4",
        "Add cards",
        "Import",
        "Fix to 1",
        "Remove",
        "Clear",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    let deck_id = view(&harness).profile.id;
    let items = view(&harness).items();
    let at = |item: Item| items.iter().position(|t| *t == item).unwrap();

    view_mut(&mut harness).item = at(Item::Fix {
        warning: 1,
        button: 0,
    });
    harness.press(KeyCode::Char(' '));
    view_mut(&mut harness).item = at(Item::Fix {
        warning: 2,
        button: 0,
    });
    harness.press(KeyCode::Char(' '));
    view_mut(&mut harness).item = at(Item::Fix {
        warning: 3,
        button: 0,
    });
    harness.press(KeyCode::Char(' '));
    assert_eq!(
        harness.pending(),
        [
            screens::Request::Deck(Request::FixQuantity {
                deck_id,
                card: sol_ring,
                quantity: 1
            }),
            screens::Request::Deck(Request::RemoveCard {
                deck_id,
                card: sol_ring
            }),
            screens::Request::Deck(Request::ClearCommander {
                deck_id,
                oathbreaker: false
            }),
        ]
    );

    harness.clear();
    harness.apply(Update::Fixed {
        deck_id,
        message: "Quantity set to 1".to_owned(),
    });
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Quantity set to 1"
    );
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(Request::Load(deck_id))]
    );

    view_mut(&mut harness).load(elves_with_short_warning());
    harness.press(KeyCode::Char('j'));
    assert_eq!(
        view(&harness).focused_item(),
        Some(Item::Fix {
            warning: 0,
            button: 1,
        })
    );
    harness.press(KeyCode::Char(' '));
    assert!(matches!(harness.screen(), Screen::DeckImport(_)));
}

fn elves_with_short_warning() -> Deck {
    let mut built = elves();
    built.warnings = vec![DeckWarning::new(
        "deck has 3 cards, commander requires at least 100",
    )];
    built
}

#[test]
fn edit_and_more_open_their_screens() {
    let mut harness = loaded(elves());
    let deck_id = view(&harness).profile.id;
    harness.press(KeyCode::Char('e'));
    assert!(
        matches!(harness.screen(), Screen::DeckForm(form) if form.mode == deck_form::Mode::Edit { deck_id })
    );
    harness.app.shared.hint = None;
    harness.press(KeyCode::Esc);
    let mut reloaded = elves();
    reloaded.deck_profile = view(&harness).profile.clone();
    harness.apply(Update::Loaded(Box::new(reloaded)));
    harness.press(KeyCode::Char('m'));
    let screen = screen_text(&harness.app);
    for expected in [
        "More actions",
        "Import cards",
        "Export cards",
        "Buy deck",
        "Clone deck",
        "Share deck",
        "Clear skips",
        "Delete deck",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::DeckExport(view) if view.deck_id == deck_id));
    assert_eq!(
        harness
            .app
            .shared
            .hint
            .as_ref()
            .map(|h| h.content.title.as_str()),
        Some("Exporting your deck")
    );
    harness.app.shared.hint = None;
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('m'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::DeckImport(view) if view.deck_id == deck_id));
    assert_eq!(
        harness
            .app
            .shared
            .hint
            .as_ref()
            .map(|h| h.content.title.as_str()),
        Some("Importing cards")
    );
}

#[test]
fn export_and_buy_wait_for_cards() {
    let mut harness = loaded(deck(deck_profile("Empty"), Vec::new()));
    let menu = view(&harness).more_menu();
    assert!(!menu.items[1].enabled && !menu.items[2].enabled);
    harness.press(KeyCode::Char('m'));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Deck(view) if view.more.is_some()));
}

#[test]
fn clone_prefills_the_name_and_waits_on_the_server() {
    let mut harness = loaded(elves());
    let deck_id = view(&harness).profile.id;
    harness.press(KeyCode::Char('m'));
    for _ in 0..3 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(view(&harness).more.is_none());
    assert!(screen_text(&harness.app).contains("Elves (clone)"));
    harness.press(KeyCode::Backspace);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(Request::Clone {
            deck_id,
            name: "Elves (clone".to_owned()
        })]
    );
    assert!(screen_text(&harness.app).contains("Cloning..."));
    harness.apply(Update::CloneFailed(
        "a deck with that name already exists".to_owned(),
    ));
    assert!(matches!(&view(&harness).dialog, Some(Dialog::Clone(c)) if !c.busy));

    let mut copy = deck_profile("Elves (clone");
    copy.format = Some(Format::Modern);
    let copy_id = copy.id;
    harness.clear();
    harness.apply(Update::Cloned {
        profile: Box::new(copy),
    });
    assert_eq!(
        harness.app.shared.toasts.items().last().unwrap().message,
        "Cloned as \"Elves (clone\""
    );
    assert_eq!(view(&harness).profile.id, copy_id);
    assert_eq!(harness.app.nav.depth(), 2, "the copy replaces the original");
}

#[test]
fn share_creates_copies_and_stops_the_link() {
    let mut harness = loaded(elves());
    let deck_id = view(&harness).profile.id;
    harness.press(KeyCode::Char('m'));
    for _ in 0..4 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(view(&harness).more.is_some(), "the sheet stays under Share");
    assert!(screen_text(&harness.app).contains("Create link"));
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Enter);
    assert!(view(&harness).more.is_none());
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(Request::Share(deck_id))]
    );
    let token = Uuid::new_v4();
    harness.apply(Update::Shared { deck_id, token });
    assert_eq!(view(&harness).share_token, Some(token));
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Share link copied"
    );
    assert_eq!(
        view(&harness).share_link().unwrap(),
        format!("{WEB_BASE}/deck/{token}")
    );

    harness.clear();
    harness.press(KeyCode::Char('m'));
    for _ in 0..4 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains("Stop share"));
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(Request::Unshare(deck_id))]
    );
    harness.apply(Update::Unshared(deck_id));
    assert_eq!(view(&harness).share_token, None);
}

#[test]
fn clear_skips_and_delete_confirm_first_and_cancel_keeps_the_sheet() {
    let mut harness = loaded(elves());
    let deck_id = view(&harness).profile.id;
    harness.press(KeyCode::Char('m'));
    for _ in 0..5 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains("This can't be undone."));
    harness.press(KeyCode::Enter);
    assert!(
        view(&harness).more.is_some(),
        "Cancel is focused and leaves the sheet"
    );
    assert_eq!(harness.pending(), []);
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(Request::ClearSkips(deck_id))]
    );
    harness.apply(Update::SkipsCleared);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Skips cleared"
    );

    harness.clear();
    harness.press(KeyCode::Char('m'));
    for _ in 0..6 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains("Are you sure you want to delete this deck?"));
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(Request::Delete(deck_id))]
    );
    harness.clear();
    harness.apply(Update::Deleted(deck_id));
    assert!(matches!(harness.screen(), Screen::Decks(_)));
}

#[test]
fn buy_offers_both_retailers_once_loaded() {
    let harness = loaded(elves());
    let dialog = view(&harness).buy_dialog();
    let labels: Vec<&str> = dialog.buttons.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(labels, ["Close", "TCGplayer ↗", "Card Kingdom ↗"]);
    assert_eq!(dialog.body, "Buy this deck's cards from a retailer.");
    let (tcg, ck) = view(&harness).buy_links().unwrap();
    assert!(tcg.starts_with("https://www.tcgplayer.com/massentry?c=1+A||"));
    assert!(ck.starts_with("https://www.cardkingdom.com/builder?c=1+A%0A"));
}

#[test]
fn a_deck_result_for_another_deck_is_ignored_and_a_failure_says_so() {
    let mut harness = Harness::on(Screen::Deck(Box::new(DeckView::opening(deck_profile(
        "Open",
    )))));
    harness.apply(Update::Loaded(Box::new(deck(
        deck_profile("Other"),
        Vec::new(),
    ))));
    assert!(view(&harness).deck.is_none());
    harness.apply(Update::Failed("deck not found".to_owned()));
    assert!(screen_text(&harness.app).contains("Could not load deck"));
}

#[test]
fn the_profile_card_names_the_oathbreaker() {
    let mut profile = deck_profile("Wrenn");
    profile.format = Some(Format::Oathbreaker);
    profile.commander_name = Some("Wrenn and Six".to_owned());
    profile.signature_spell_name = Some("Crop Rotation".to_owned());
    let harness = Harness::on(Screen::Deck(Box::new(DeckView::opening(profile))));
    let screen = screen_text(&harness.app);
    for expected in ["Profile", "Oathbreaker", "Wrenn and Six", "Signature spell"] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_deck_screen_fits_a_tiny_terminal() {
    let mut harness = loaded(elves());
    render(&harness.app, 12, 5);
    render(&harness.app, 1, 1);
    harness.press(KeyCode::Char('m'));
    render(&harness.app, 12, 5);
    view_mut(&mut harness).dialog = Some(Dialog::Clone(CloneDialog::new("A")));
    render(&harness.app, 12, 5);
}

#[test]
fn c_opens_the_deck_cards_view_on_this_deck() {
    let mut harness = loaded(elves());
    let deck_id = view(&harness).profile.id;
    harness.press(KeyCode::Char('c'));
    let Screen::DeckCards(cards) = harness.screen() else {
        panic!("not on the deck cards view: {:?}", harness.screen());
    };
    assert_eq!(cards.deck_id, deck_id);
    assert!(harness.pending().contains(&screens::Request::DeckCards(
        screens::deck_cards::Request::Load { deck_id }
    )));
}

#[test]
fn add_cards_on_the_below_minimum_warning_opens_add_on_this_deck() {
    let mut harness = loaded(elves_with_short_warning());
    let deck_id = view(&harness).profile.id;
    view_mut(&mut harness).focus_on(Pane::Section(Section::Warnings));
    assert_eq!(
        view(&harness).focused_item(),
        Some(Item::Fix {
            warning: 0,
            button: 0,
        })
    );
    harness.press(KeyCode::Enter);
    assert!(view(&harness).zoomed, "Enter zooms first");
    harness.press(KeyCode::Enter);
    assert_eq!(harness.swipe_view().deck.id, deck_id);
}

/// Elves with a ramp role carrying a tag, an oracle tag on the profile, a price target, and a copy-limit warning: every section shows.
fn everything() -> Deck {
    let elves = elves();
    let mut ramp = card("Llanowar Elves");
    ramp.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    ramp.scryfall_data.cmc = Some(1.0);
    ramp.card_profile.card_roles = vec!["ramp".to_owned()];
    ramp.card_profile
        .oracle_tags_by_role
        .insert("ramp".to_owned(), vec!["mana-dork".to_owned()]);
    let mut profile = elves.deck_profile.clone();
    profile.oracle_tags = vec!["mana-dork".to_owned()];
    profile.price_target = Some(50.0);
    profile.card_count = 4;
    let mut cards: Vec<_> = elves.entries.into_iter().map(|e| e.card).collect();
    for card in &mut cards {
        card.card_profile.card_roles.clear();
        card.card_profile.oracle_tags_by_role.clear();
    }
    cards.push(ramp);
    let mut built = deck(profile, cards);
    let first = built.entries[0].card.scryfall_data.id;
    built.warnings = vec![DeckWarning::with_action(
        "A exceeds copy limit (2/1)",
        first,
        WarningAction::FixQuantity(1),
    )];
    built
}

#[test]
fn every_section_shows_and_the_screen_fits_every_size() {
    let mut harness = loaded(everything());
    assert_eq!(
        view(&harness).panes(),
        [
            Pane::Profile,
            Pane::Section(Section::Budget),
            Pane::Section(Section::Tags),
            Pane::Section(Section::Distributions),
            Pane::Section(Section::Mana),
            Pane::Section(Section::DrawOdds),
            Pane::Section(Section::Warnings),
        ]
    );
    for (width, height) in [(200, 50), (140, 40), (80, 24), (12, 5), (1, 1)] {
        for pane in view(&harness).panes() {
            view_mut(&mut harness).focus_on(pane);
            for zoomed in [false, true] {
                view_mut(&mut harness).zoomed = zoomed;
                render(&harness.app, width, height);
            }
        }
    }
}

#[test]
fn wide_shows_the_profile_beside_every_section_at_once() {
    let harness = loaded(everything());
    let screen = text_at(&harness, 200, 50);
    for expected in [
        "Profile",
        "Elves",
        "Total price",
        "Mana curve",
        "Types",
        "Roles",
        "Odds of ≥1 in opening hand",
        "Oracle tags",
        "Warnings 1",
        "e Edit",
        "Esc Back",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn medium_shows_the_profile_beside_one_section_tabbed() {
    let mut harness = loaded(everything());
    let screen = text_at(&harness, 140, 40);
    for expected in [
        "Elves",
        " Budget ",
        " Tags 1 ",
        " Draw odds ",
        "Total price",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    assert!(!screen.contains("Mana curve"), "one section at a time");
    for _ in 0..3 {
        harness.press(KeyCode::Tab);
    }
    let screen = text_at(&harness, 140, 40);
    assert!(screen.contains("Roles") && screen.contains("Elves"));
    assert!(!screen.contains("Total price"));
}

#[test]
fn narrow_tabs_the_profile_first_and_tab_walks_the_rest() {
    let mut harness = loaded(everything());
    let screen = text_at(&harness, 80, 24);
    assert!(screen.contains(" Profile ") && screen.contains("Elves"));
    assert!(!screen.contains("Total price"));
    harness.press(KeyCode::Tab);
    let screen = text_at(&harness, 80, 24);
    assert!(screen.contains("Total price"));
    assert!(!screen.contains("Elves"), "the profile's tab is hidden");
    for key in ["Tab", "Enter", "e Edit", "m More", "Esc Back"] {
        assert!(screen.contains(key), "the hints keep {key}");
    }
}

#[test]
fn every_sections_keys_work_in_every_layout() {
    for (width, height) in SIZES {
        let mut harness = loaded(everything());
        let size = format!("{width}x{height}");

        harness.press(KeyCode::Tab);
        harness.press(KeyCode::Char('l'));
        assert_eq!(view(&harness).currency, PriceCurrency::Eur, "{size}");
        assert!(text_at(&harness, width, height).contains('€'), "{size}");

        harness.press(KeyCode::Tab);
        harness.press(KeyCode::Char('i'));
        assert!(
            text_at(&harness, width, height).contains("Deck tags: the archetypes"),
            "{size}"
        );
        harness.press(KeyCode::Esc);
        assert!(view(&harness).dialog.is_none(), "{size}");

        harness.press(KeyCode::Tab);
        assert_eq!(
            view(&harness).focused_item(),
            Some(Item::Role("ramp".to_owned())),
            "{size}"
        );
        harness.press(KeyCode::Char(' '));
        harness.press(KeyCode::Char('j'));
        harness.press(KeyCode::Char(' '));
        assert_eq!(
            view(&harness).tag_open.as_deref(),
            Some("mana-dork"),
            "{size}"
        );
        harness.press(KeyCode::Char('j'));
        assert_eq!(
            view(&harness).focused_item(),
            Some(Item::Examples("mana-dork".to_owned())),
            "{size}"
        );
        let screen = text_at(&harness, width, height);
        for expected in ["mana-dork", "No description yet", "Examples"] {
            assert!(screen.contains(expected), "{size}: missing {expected}");
        }
        harness.press(KeyCode::Char(' '));
        assert!(
            harness
                .app
                .shared
                .toasts
                .items()
                .iter()
                .any(|t| t.message.contains("Oracle tag examples")),
            "{size}"
        );
        // The toast sits over the top-left corner; let it go before reading the panels.
        harness
            .app
            .shared
            .toasts
            .prune(std::time::Instant::now() + std::time::Duration::from_secs(60));

        harness.press(KeyCode::Tab);
        assert!(
            text_at(&harness, width, height).contains("Mana curve"),
            "{size}"
        );
        harness.press(KeyCode::Char('i'));
        assert!(
            text_at(&harness, width, height).contains("Mana cost fulfillment: per color"),
            "{size}"
        );
        harness.press(KeyCode::Esc);

        harness.press(KeyCode::Tab);
        harness.press(KeyCode::Char('+'));
        harness.press(KeyCode::Char('l'));
        assert_eq!(view(&harness).turn, 2, "{size}");
        assert!(
            text_at(&harness, width, height).contains("Odds of ≥1 by turn 2"),
            "{size}"
        );

        harness.press(KeyCode::Tab);
        assert!(
            text_at(&harness, width, height).contains("Fix to 1"),
            "{size}"
        );
        harness.clear();
        harness.press(KeyCode::Char(' '));
        assert!(
            matches!(
                harness.pending(),
                [screens::Request::Deck(Request::FixQuantity {
                    quantity: 1,
                    ..
                })]
            ),
            "{size}"
        );
    }
}

#[test]
fn enter_zooms_the_focused_pane_and_esc_unzooms_before_going_back() {
    let mut harness = loaded(everything());
    for _ in 0..4 {
        harness.press(KeyCode::Tab);
    }
    assert_eq!(view(&harness).focused(), Pane::Section(Section::Mana));
    harness.press(KeyCode::Enter);
    assert!(view(&harness).zoomed);
    for (width, height) in SIZES {
        let screen = text_at(&harness, width, height);
        assert!(screen.contains("Mana curve"), "{width}x{height}");
        assert!(!screen.contains("Total price"), "{width}x{height}");
        assert!(!screen.contains("Elves"), "{width}x{height}");
        assert!(screen.contains("Esc unzoom"), "{width}x{height}");
    }
    harness.press(KeyCode::Tab);
    assert!(
        text_at(&harness, 200, 50).contains("Odds of ≥1"),
        "Tab moves the zoom to the next pane"
    );
    harness.press(KeyCode::Esc);
    assert!(!view(&harness).zoomed);
    assert!(matches!(harness.screen(), Screen::Deck(_)));
    assert!(text_at(&harness, 200, 50).contains("Total price"));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Decks(_)));
}

#[test]
fn a_reload_keeps_the_focus_on_the_stats_through_the_skeleton() {
    let mut harness = loaded(everything());
    view_mut(&mut harness).focus_on(Pane::Section(Section::DrawOdds));
    harness.press(KeyCode::Char('r'));
    assert_eq!(view(&harness).focused(), Pane::Loading);
    for (width, height) in SIZES {
        assert!(text_at(&harness, width, height).contains('░'));
    }
    let mut reloaded = everything();
    reloaded.deck_profile = view(&harness).profile.clone();
    harness.apply(Update::Loaded(Box::new(reloaded)));
    assert_eq!(view(&harness).focused(), Pane::Section(Section::DrawOdds));
}

#[test]
fn a_short_pane_with_no_items_scrolls_to_its_last_row() {
    let mut harness = loaded(everything());
    view_mut(&mut harness).focus_on(Pane::Section(Section::DrawOdds));
    harness.press(KeyCode::Enter);
    assert!(!text_at(&harness, 80, 10).contains("RAMP"));
    for _ in 0..10 {
        harness.press(KeyCode::Char('j'));
    }
    assert_eq!(view(&harness).scroll, view(&harness).scroll_limit());
    assert!(text_at(&harness, 80, 10).contains("RAMP"));
    harness.press(KeyCode::Char('k'));
    assert_eq!(
        view(&harness).scroll,
        view(&harness).scroll_limit().saturating_sub(1)
    );
    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).scroll, 0, "a new pane starts at its top");
}
