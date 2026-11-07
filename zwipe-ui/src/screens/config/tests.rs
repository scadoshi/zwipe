use super::*;
use crate::toast::{NORMAL, QUICK, ToastKind};

fn slugs(items: &[&str]) -> Vec<String> {
    items.iter().map(ToString::to_string).collect()
}

fn gruvbox(dark_mode: bool) -> Theme {
    Theme {
        name: "gruvbox".to_owned(),
        dark_mode,
    }
}

/// The account's settings: gruvbox dark, themed pips, Universes Beyond hidden with one exception.
fn account() -> UserSettings {
    UserSettings {
        theme: "gruvbox".to_owned(),
        dark_mode: true,
        exclude_universes_beyond: true,
        universes_beyond_exceptions: slugs(&["a"]),
        pip_style: PipStyle::Themed,
        swipe_confirmations: SwipeConfirmations::Chip,
    }
}

/// A state whose fetch is out, with the ticket it holds.
fn loading() -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load);
    let [
        Effect::OfferHint(OneTimeHint::Config),
        Effect::FetchSettings(ticket),
    ] = effects.as_slice()
    else {
        panic!("Load offers the hint and fetches: {effects:?}");
    };
    (state, *ticket)
}

/// A state with `settings` loaded.
fn loaded(settings: UserSettings) -> State {
    let (mut state, ticket) = loading();
    let _ = state.update(Action::Loaded {
        ticket,
        settings: Box::new(settings),
    });
    state
}

/// The ticket and change of the one [`Effect::Save`] in `effects`.
fn save_in(effects: &[Effect]) -> (SaveTicket, SettingChange) {
    let mut saves = effects.iter().filter_map(|effect| match effect {
        Effect::Save { ticket, change } => Some((*ticket, change.clone())),
        _ => None,
    });
    let (Some(save), None) = (saves.next(), saves.next()) else {
        panic!("one save: {effects:?}");
    };
    save
}

fn saved(state: &mut State, ticket: SaveTicket, settings: UserSettings) -> Vec<Effect> {
    state.update(Action::Saved {
        ticket,
        settings: Box::new(settings),
    })
}

/// A state with the Exceptions sheet open on the saved `["a"]`.
fn sheet_open() -> State {
    let mut state = loaded(account());
    let _ = state.update(Action::OpenExceptions);
    state
}

#[test]
fn a_new_state_is_loading_with_universes_beyond_off() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert!(!state.universes_beyond_enabled());
    assert!(!state.exceptions_enabled());
    assert!(!state.hides_universes_beyond());
    assert_eq!(state.universes_beyond_label(), SHOW);
    assert!(state.exceptions_muted());
    assert!(!state.shows_exception_chips());
    assert!(!state.editing_exceptions());
    assert_eq!(state.draft(), [""; 0]);
    assert!(state.draft_unchanged());
}

#[test]
fn load_offers_the_hint_then_fetches() {
    let (state, _) = loading();
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn the_hint_is_offered_once_per_visit() {
    let (mut state, _) = loading();
    let effects = state.update(Action::Load);
    assert!(
        matches!(effects.as_slice(), [Effect::FetchSettings(_)]),
        "{effects:?}"
    );
}

#[test]
fn loaded_settings_enable_universes_beyond_and_show_the_pips() {
    let (mut state, ticket) = loading();
    let mut settings = account();
    settings.pip_style = PipStyle::Classic;
    let effects = state.update(Action::Loaded {
        ticket,
        settings: Box::new(settings),
    });
    assert_eq!(effects, [Effect::ShowPips(PipStyle::Classic)]);
    assert_eq!(state.status(), Status::Loaded);
    assert!(state.universes_beyond_enabled());
    assert!(state.exceptions_enabled());
    assert!(state.hides_universes_beyond());
    assert_eq!(state.universes_beyond_label(), HIDE);
    assert!(!state.exceptions_muted());
    assert_eq!(state.exceptions(), slugs(&["a"]));
    assert!(state.shows_exception_chips());
}

#[test]
fn exceptions_stay_off_while_universes_beyond_shows() {
    let mut settings = account();
    settings.exclude_universes_beyond = false;
    let mut state = loaded(settings);
    assert!(state.universes_beyond_enabled());
    assert!(!state.exceptions_enabled());
    assert!(state.exceptions_muted());
    assert_eq!(state.update(Action::OpenExceptions), []);
    assert!(!state.editing_exceptions());
}

#[test]
fn a_failed_load_keeps_universes_beyond_off_and_says_nothing() {
    let (mut state, ticket) = loading();
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert_eq!(state.status(), Status::Failed);
    assert!(!state.universes_beyond_enabled());
    assert_eq!(state.update(Action::ToggleUniversesBeyond), []);
    assert_eq!(state.update(Action::OpenExceptions), []);
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, old) = loading();
    let effects = state.update(Action::Load);
    let [Effect::FetchSettings(new)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(
        state.update(Action::Loaded {
            ticket: old,
            settings: Box::new(account()),
        }),
        []
    );
    assert_eq!(state.update(Action::Failed { ticket: old }), []);
    assert_eq!(state.status(), Status::Loading);
    let _ = state.update(Action::Loaded {
        ticket: *new,
        settings: Box::new(account()),
    });
    assert_eq!(state.status(), Status::Loaded);
}

#[test]
fn a_failed_reload_keeps_what_loaded() {
    let mut state = loaded(account());
    let effects = state.update(Action::Load);
    let [Effect::FetchSettings(ticket)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(state.status(), Status::Loaded);
    let _ = state.update(Action::Failed { ticket: *ticket });
    assert_eq!(state.status(), Status::Loaded);
    assert!(state.hides_universes_beyond());
}

#[test]
fn dark_mode_shows_at_once_and_saves_the_same_theme() {
    let mut state = State::new();
    let effects = state.update(Action::ToggleDarkMode {
        shown: gruvbox(true),
    });
    assert_eq!(effects.first(), Some(&Effect::ShowTheme(gruvbox(false))));
    let (_, change) = save_in(&effects);
    assert_eq!(
        change,
        SettingChange::DarkMode {
            theme: "gruvbox".to_owned(),
            dark_mode: false,
        }
    );
}

#[test]
fn a_saved_dark_mode_draws_the_reply_without_a_toast() {
    let mut state = State::new();
    let effects = state.update(Action::ToggleDarkMode {
        shown: gruvbox(true),
    });
    let (ticket, _) = save_in(&effects);
    let mut reply = account();
    reply.dark_mode = false;
    assert_eq!(
        saved(&mut state, ticket, reply),
        [Effect::ThemeSaved(gruvbox(false))]
    );
}

#[test]
fn a_failed_dark_mode_puts_back_the_mode_before_the_tap() {
    let mut state = State::new();
    let effects = state.update(Action::ToggleDarkMode {
        shown: gruvbox(true),
    });
    let (ticket, _) = save_in(&effects);
    assert_eq!(
        state.update(Action::SaveFailed { ticket }),
        [Effect::ShowTheme(gruvbox(true))]
    );
}

#[test]
fn a_dark_mode_reply_overtaken_by_a_newer_tap_is_ignored() {
    let mut state = State::new();
    let first = save_in(&state.update(Action::ToggleDarkMode {
        shown: gruvbox(true),
    }))
    .0;
    let second = save_in(&state.update(Action::ToggleDarkMode {
        shown: gruvbox(false),
    }))
    .0;
    assert_eq!(saved(&mut state, first, account()), []);
    assert_eq!(
        state.update(Action::SaveFailed { ticket: second }),
        [Effect::ShowTheme(gruvbox(false))]
    );
}

#[test]
fn pips_show_at_once_and_save_the_other_style() {
    let mut state = State::new();
    let effects = state.update(Action::TogglePips {
        shown: PipStyle::Themed,
    });
    assert_eq!(effects.first(), Some(&Effect::ShowPips(PipStyle::Classic)));
    let (ticket, change) = save_in(&effects);
    assert_eq!(change, SettingChange::Pips(PipStyle::Classic));
    let mut reply = account();
    reply.pip_style = PipStyle::Classic;
    assert_eq!(
        saved(&mut state, ticket, reply),
        [
            Effect::PipsSaved(PipStyle::Classic),
            Effect::Toast(Toast::new(
                ToastKind::Success,
                "Pips in classic colors",
                NORMAL
            )),
        ]
    );
}

#[test]
fn a_failed_pips_save_puts_back_the_style_before_the_tap() {
    let mut state = State::new();
    let (ticket, _) = save_in(&state.update(Action::TogglePips {
        shown: PipStyle::Classic,
    }));
    assert_eq!(
        state.update(Action::SaveFailed { ticket }),
        [Effect::ShowPips(PipStyle::Classic)]
    );
}

#[test]
fn a_pips_reply_overtaken_by_a_newer_tap_is_ignored() {
    let mut state = State::new();
    let (first, _) = save_in(&state.update(Action::TogglePips {
        shown: PipStyle::Themed,
    }));
    let _ = state.update(Action::TogglePips {
        shown: PipStyle::Classic,
    });
    assert_eq!(state.update(Action::SaveFailed { ticket: first }), []);
}

#[test]
fn universes_beyond_flips_at_once_and_toasts_the_reply() {
    let mut settings = account();
    settings.exclude_universes_beyond = false;
    let mut state = loaded(settings);
    let effects = state.update(Action::ToggleUniversesBeyond);
    assert!(state.hides_universes_beyond());
    assert!(state.exceptions_enabled());
    let (ticket, change) = save_in(&effects);
    assert_eq!(change, SettingChange::HideUniversesBeyond(true));
    assert_eq!(
        saved(&mut state, ticket, account()),
        [Effect::Toast(Toast::new(
            ToastKind::Success,
            "Universes Beyond hidden",
            NORMAL
        ))]
    );
    assert!(state.hides_universes_beyond());
}

#[test]
fn universes_beyond_shows_what_the_server_answered() {
    let mut state = loaded(account());
    let (ticket, _) = save_in(&state.update(Action::ToggleUniversesBeyond));
    assert!(!state.hides_universes_beyond());
    let effects = saved(&mut state, ticket, account());
    assert!(state.hides_universes_beyond(), "the reply still hides them");
    assert_eq!(
        effects,
        [Effect::Toast(Toast::new(
            ToastKind::Success,
            "Universes Beyond hidden",
            NORMAL
        ))]
    );
}

#[test]
fn a_failed_universes_beyond_save_puts_back_the_value_before_the_tap() {
    let mut state = loaded(account());
    let (_, _) = save_in(&state.update(Action::ToggleUniversesBeyond));
    let (second, _) = save_in(&state.update(Action::ToggleUniversesBeyond));
    assert!(state.hides_universes_beyond());
    assert_eq!(state.update(Action::SaveFailed { ticket: second }), []);
    assert!(
        !state.hides_universes_beyond(),
        "puts back the value before the second tap"
    );
}

#[test]
fn a_universes_beyond_reply_overtaken_by_a_newer_tap_is_ignored() {
    let mut state = loaded(account());
    let (first, _) = save_in(&state.update(Action::ToggleUniversesBeyond));
    let _ = state.update(Action::ToggleUniversesBeyond);
    let mut reply = account();
    reply.exclude_universes_beyond = false;
    assert_eq!(saved(&mut state, first, reply), []);
    assert!(state.hides_universes_beyond());
}

#[test]
fn a_tap_on_one_toggle_leaves_anothers_reply_current() {
    let mut state = loaded(account());
    let (pips, _) = save_in(&state.update(Action::TogglePips {
        shown: PipStyle::Themed,
    }));
    let _ = state.update(Action::ToggleUniversesBeyond);
    let _ = state.update(Action::ToggleDarkMode {
        shown: gruvbox(true),
    });
    assert_eq!(
        state.update(Action::SaveFailed { ticket: pips }),
        [Effect::ShowPips(PipStyle::Themed)]
    );
}

#[test]
fn a_reply_lands_once() {
    let mut state = State::new();
    let (ticket, _) = save_in(&state.update(Action::TogglePips {
        shown: PipStyle::Themed,
    }));
    let _ = state.update(Action::SaveFailed { ticket });
    assert_eq!(saved(&mut state, ticket, account()), []);
}

#[test]
fn change_opens_the_themes_sheet() {
    assert_eq!(
        State::new().update(Action::OpenThemes),
        [Effect::OpenSheet(Sheet::Themes)]
    );
}

#[test]
fn a_kept_theme_saves_without_closing_the_sheet_or_a_preview_change() {
    let mut state = State::new();
    let dracula = Theme {
        name: "dracula".to_owned(),
        dark_mode: false,
    };
    let effects = state.update(Action::KeepTheme(dracula.clone()));
    assert_eq!(effects.len(), 1, "{effects:?}");
    let (ticket, change) = save_in(&effects);
    assert_eq!(
        change,
        SettingChange::Theme {
            theme: "dracula".to_owned(),
            dark_mode: false,
        }
    );
    let mut reply = account();
    reply.theme = "dracula".to_owned();
    reply.dark_mode = false;
    assert_eq!(
        saved(&mut state, ticket, reply),
        [Effect::ThemeSaved(dracula)]
    );
}

#[test]
fn a_failed_theme_save_changes_nothing() {
    let mut state = State::new();
    let (ticket, _) = save_in(&state.update(Action::KeepTheme(gruvbox(false))));
    assert_eq!(state.update(Action::SaveFailed { ticket }), []);
}

#[test]
fn every_theme_save_lands() {
    let mut state = State::new();
    let (first, _) = save_in(&state.update(Action::KeepTheme(gruvbox(false))));
    let (second, _) = save_in(&state.update(Action::KeepTheme(gruvbox(true))));
    assert_eq!(saved(&mut state, second, account()).len(), 1);
    assert_eq!(saved(&mut state, first, account()).len(), 1);
}

#[test]
fn the_exceptions_sheet_opens_on_the_saved_list_and_offers_its_hint_once() {
    let mut state = loaded(account());
    assert_eq!(
        state.update(Action::OpenExceptions),
        [
            Effect::OpenSheet(Sheet::Exceptions),
            Effect::OfferHint(OneTimeHint::UniversesBeyondExceptions),
        ]
    );
    assert!(state.editing_exceptions());
    assert_eq!(state.draft(), slugs(&["a"]));
    assert!(state.draft_unchanged());
    let _ = state.update(Action::ToggleException("b".to_owned()));
    let _ = state.update(Action::DiscardExceptions);
    assert_eq!(
        state.update(Action::OpenExceptions),
        [Effect::OpenSheet(Sheet::Exceptions)]
    );
    assert_eq!(
        state.draft(),
        slugs(&["a"]),
        "a reopened sheet starts from the saved list"
    );
}

#[test]
fn the_draft_toggles_and_clears() {
    let mut state = sheet_open();
    let _ = state.update(Action::ToggleException("b".to_owned()));
    assert_eq!(state.draft(), slugs(&["a", "b"]));
    assert!(state.is_drafted("b"));
    assert!(!state.draft_unchanged());
    let _ = state.update(Action::ToggleException("a".to_owned()));
    assert_eq!(state.draft(), slugs(&["b"]));
    assert!(!state.is_drafted("a"));
    let _ = state.update(Action::ClearExceptions);
    assert_eq!(state.draft(), [""; 0]);
    assert!(!state.draft_unchanged());
}

#[test]
fn a_draft_back_on_the_saved_set_is_unchanged() {
    let mut state = sheet_open();
    let _ = state.update(Action::ToggleException("a".to_owned()));
    let _ = state.update(Action::ToggleException("a".to_owned()));
    assert!(state.draft_unchanged());
}

#[test]
fn the_draft_ignores_edits_while_the_sheet_is_closed() {
    let mut state = loaded(account());
    assert_eq!(state.update(Action::ToggleException("b".to_owned())), []);
    assert_eq!(state.update(Action::ClearExceptions), []);
    assert_eq!(state.update(Action::SaveExceptions), []);
    assert_eq!(state.update(Action::DiscardExceptions), []);
    assert_eq!(state.draft(), [""; 0]);
}

#[test]
fn discarding_a_changed_draft_says_so_before_closing() {
    let mut state = sheet_open();
    let _ = state.update(Action::ToggleException("b".to_owned()));
    assert_eq!(
        state.update(Action::DiscardExceptions),
        [
            Effect::Toast(Toast::new(ToastKind::Info, "Exceptions unchanged", QUICK)),
            Effect::CloseSheet(Sheet::Exceptions),
        ]
    );
    assert!(!state.editing_exceptions());
    assert_eq!(
        state.draft(),
        slugs(&["a", "b"]),
        "a closing sheet keeps its picks"
    );
    assert_eq!(state.update(Action::ToggleException("c".to_owned())), []);
    assert_eq!(state.draft(), slugs(&["a", "b"]));
    assert_eq!(state.exceptions(), slugs(&["a"]));
}

#[test]
fn discarding_an_unchanged_draft_closes_quietly() {
    let mut state = sheet_open();
    assert_eq!(
        state.update(Action::DiscardExceptions),
        [Effect::CloseSheet(Sheet::Exceptions)]
    );
}

#[test]
fn saved_exceptions_close_the_sheet_then_land_on_the_reply() {
    let mut state = sheet_open();
    let _ = state.update(Action::ToggleException("b".to_owned()));
    let effects = state.update(Action::SaveExceptions);
    assert_eq!(
        effects.first(),
        Some(&Effect::CloseSheet(Sheet::Exceptions))
    );
    let (ticket, change) = save_in(&effects);
    assert_eq!(change, SettingChange::Exceptions(slugs(&["a", "b"])));
    assert!(!state.editing_exceptions());
    assert_eq!(state.exceptions(), slugs(&["a"]), "waits for the server");
    let mut reply = account();
    reply.universes_beyond_exceptions = slugs(&["b", "a"]);
    assert_eq!(
        saved(&mut state, ticket, reply),
        [Effect::Toast(Toast::new(
            ToastKind::Success,
            "Exceptions saved",
            QUICK
        ))]
    );
    assert_eq!(state.exceptions(), slugs(&["b", "a"]));
}

#[test]
fn a_failed_exceptions_save_keeps_the_saved_list() {
    let mut state = sheet_open();
    let _ = state.update(Action::ClearExceptions);
    let (ticket, _) = save_in(&state.update(Action::SaveExceptions));
    assert_eq!(state.update(Action::SaveFailed { ticket }), []);
    assert_eq!(state.exceptions(), slugs(&["a"]));
}

#[test]
fn unknown_saved_slugs_still_draw_the_chip_row_but_no_chip() {
    let mut settings = account();
    settings.universes_beyond_exceptions = slugs(&["not-a-franchise"]);
    let state = loaded(settings);
    assert!(state.shows_exception_chips());
    assert_eq!(state.exception_names(), Vec::<&str>::new());
}

#[test]
fn back_goes_to_the_screen_before() {
    assert_eq!(
        State::new().update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Settings");
    assert_eq!(APPEARANCE, "Appearance");
    assert_eq!(CARDS, "Cards");
    assert_eq!(THEME, "Theme");
    assert_eq!(PIPS, "Pips");
    assert_eq!(UNIVERSES_BEYOND, "Universes Beyond");
    assert_eq!(EXCEPTIONS, "Exceptions");
    assert_eq!(CHANGE, "Change");
    assert_eq!(HIDE, "Hide");
    assert_eq!(SHOW, "Show");
    assert_eq!(BACK, "Back");
    assert_eq!(SAVE, "Save");
    assert_eq!(pips_label(PipStyle::Themed), "Themed");
    assert_eq!(pips_label(PipStyle::Classic), "Classic");
    assert_eq!(selected_line(3), "3 selected");
}

#[test]
fn swipe_confirmations_cycle_save_and_keep_the_saved_value() {
    let (mut idle, _) = loading();
    assert_eq!(
        idle.update(Action::CycleSwipeConfirmations),
        [],
        "nothing to cycle before the settings load"
    );

    let mut state = loaded(account());
    assert_eq!(state.swipe_confirmations(), SwipeConfirmations::Chip);
    let effects = state.update(Action::CycleSwipeConfirmations);
    let (ticket, change) = save_in(&effects);
    assert_eq!(
        change,
        SettingChange::SwipeConfirmations(SwipeConfirmations::Toasts)
    );
    assert_eq!(state.swipe_confirmations(), SwipeConfirmations::Toasts);

    let mut saved = account();
    saved.swipe_confirmations = SwipeConfirmations::Toasts;
    let effects = state.update(Action::Saved {
        ticket,
        settings: Box::new(saved),
    });
    assert!(effects.contains(&Effect::SwipeConfirmationsSaved(SwipeConfirmations::Toasts)));
    assert_eq!(config_label(&state), "Toasts");
}

#[test]
fn a_failed_swipe_confirmations_save_puts_the_value_back() {
    let mut state = loaded(account());
    let (ticket, _) = save_in(&state.update(Action::CycleSwipeConfirmations));
    let _ = state.update(Action::SaveFailed { ticket });
    assert_eq!(state.swipe_confirmations(), SwipeConfirmations::Chip);
}

/// The Swipe confirmations button's text in `state`.
fn config_label(state: &State) -> &'static str {
    swipe_confirmations_label(state.swipe_confirmations())
}
