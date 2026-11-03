use chrono::Duration as Span;
use uuid::Uuid;
use zwipe_core::domain::user::Email;

use super::*;

fn user(verified: bool) -> User {
    let mut user = User::new(
        Uuid::nil(),
        Username::new("scotty").unwrap(),
        Email::new("scotty@zwipe.net").unwrap(),
    );
    if verified {
        user.email_verified_at = Some(Utc::now());
    }
    user
}

/// A screen opened for an unverified user, with the open's effects drained.
fn opened() -> State {
    let mut state = State::new();
    let _ = state.update(Action::Load(Some(Box::new(user(false)))));
    state
}

/// Nothing, as an empty list of whatever is compared.
fn none<T>() -> Vec<T> {
    Vec::new()
}

fn ticket_of(effects: &[Effect]) -> Ticket {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::FetchUser(t)
            | Effect::CheckVerification(t)
            | Effect::ResendVerification(t)
            | Effect::Wait(t)
            | Effect::ChangeUsername { ticket: t, .. }
            | Effect::ChangeEmail { ticket: t, .. }
            | Effect::ChangePassword { ticket: t, .. }
            | Effect::DeleteAccount { ticket: t, .. } => Some(*t),
            _ => None,
        })
        .expect("an effect holding a ticket")
}

fn wait_of(effects: &[Effect]) -> Ticket {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::Wait(t) => Some(*t),
            _ => None,
        })
        .expect("a wait")
}

fn edit(state: &mut State, sheet: Sheet, field: Field, value: &str) {
    let effects = state.update(Action::Edit {
        sheet,
        field,
        value: Typed::new(value),
    });
    assert_eq!(effects, none(), "typing asks for nothing");
}

/// Ticks `wait` until the count stops asking for another, returning how many ticks it took.
fn run_down(state: &mut State, wait: Ticket) -> usize {
    let mut ticks: usize = 0;
    loop {
        ticks = ticks.checked_add(1).unwrap();
        let effects = state.update(Action::Tick { ticket: wait });
        if effects.is_empty() {
            return ticks;
        }
        assert_eq!(effects, vec![Effect::Wait(wait)], "tick {ticks}");
    }
}

#[test]
fn a_new_screen_has_no_user_and_nothing_out() {
    let state = State::new();
    assert_eq!(state.user(), None);
    assert_eq!(state.rows(), none());
    assert!(!state.is_sending() && !state.is_checking() && !state.is_deleting());
    assert!(state.can_resend());
    assert!(!state.can_delete(), "no dialog is open");
}

#[test]
fn load_offers_the_hint_then_refreshes_the_user() {
    let mut state = State::new();
    let effects = state.update(Action::Load(Some(Box::new(user(false)))));
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        vec![
            Effect::OfferHint(OneTimeHint::Profile),
            Effect::FetchUser(ticket)
        ]
    );
    assert_eq!(state.user(), Some(&user(false)));

    let again = state.update(Action::Load(Some(Box::new(user(false)))));
    assert!(
        !again.contains(&Effect::OfferHint(OneTimeHint::Profile)),
        "the hint is offered once per visit"
    );
}

#[test]
fn load_without_a_session_offers_the_hint_and_fetches_nothing() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Load(None)),
        vec![Effect::OfferHint(OneTimeHint::Profile)]
    );
    assert_eq!(state.rows(), none());
}

#[test]
fn the_refreshed_user_replaces_the_session_s_copy() {
    let mut state = State::new();
    let ticket = ticket_of(&state.update(Action::Load(Some(Box::new(user(false))))));
    let effects = state.update(Action::UserLoaded {
        ticket,
        user: Box::new(user(true)),
    });
    assert_eq!(effects, none(), "the open's refresh raises no toast");
    assert!(state.is_verified());
    assert_eq!(state.badge(), VERIFIED);
}

#[test]
fn a_failed_refresh_keeps_the_session_s_user() {
    let mut state = State::new();
    let ticket = ticket_of(&state.update(Action::Load(Some(Box::new(user(false))))));
    assert_eq!(state.update(Action::Failed { ticket }), none());
    assert_eq!(state.user(), Some(&user(false)));
}

#[test]
fn a_late_refresh_is_ignored() {
    let mut state = State::new();
    let old = ticket_of(&state.update(Action::Load(Some(Box::new(user(false))))));
    let _ = state.update(Action::Load(Some(Box::new(user(false)))));
    let _ = state.update(Action::UserLoaded {
        ticket: old,
        user: Box::new(user(true)),
    });
    assert!(!state.is_verified(), "a reply to an earlier open landed");
}

#[test]
fn rows_follow_verification_and_the_pending_address() {
    let mut state = opened();
    assert_eq!(
        state.rows(),
        vec![
            Row::Username,
            Row::Email,
            Row::Verification,
            Row::Password,
            Row::Website,
            Row::PrivacyPolicy,
            Row::Version,
        ]
    );
    assert_eq!(state.badge(), UNVERIFIED);

    let mut pending = user(true);
    pending.pending_email = Some(Email::new("new@zwipe.net").unwrap());
    let _ = state.update(Action::Load(Some(Box::new(pending))));
    assert_eq!(
        state.rows(),
        vec![
            Row::Username,
            Row::Email,
            Row::Pending,
            Row::Password,
            Row::Website,
            Row::PrivacyPolicy,
            Row::Version,
        ]
    );
    let lists: Vec<&str> = state.rows().into_iter().map(Row::list).collect();
    assert_eq!(
        lists,
        vec![ACCOUNT, ACCOUNT, ACCOUNT, ACCOUNT, ABOUT, ABOUT, ABOUT]
    );
}

#[test]
fn the_pending_row_carries_its_note_when_the_server_sent_an_expiry() {
    let now = Utc::now();
    let mut pending = user(false);
    pending.pending_email = Some(Email::new("new@zwipe.net").unwrap());
    let mut state = State::new();
    let _ = state.update(Action::Load(Some(Box::new(pending.clone()))));
    assert_eq!(
        state.pending(now),
        Some(Pending {
            email: "new@zwipe.net".to_owned(),
            note: None,
        }),
        "an older server sends no expiry"
    );

    pending.pending_email_expires_at = Some(now + Span::minutes(30));
    let _ = state.update(Action::Load(Some(Box::new(pending))));
    assert_eq!(
        state.pending(now).and_then(|p| p.note).as_deref(),
        Some("Link expires in 30 minutes")
    );
    assert_eq!(opened().pending(now), None);
}

#[test]
fn a_check_toasts_what_it_found() {
    let mut state = opened();
    let effects = state.update(Action::CheckVerification);
    let ticket = ticket_of(&effects);
    assert_eq!(effects, vec![Effect::CheckVerification(ticket)]);
    assert!(state.is_checking());
    assert_eq!(
        state.update(Action::CheckVerification),
        none(),
        "a second check waits for the first"
    );

    let effects = state.update(Action::UserLoaded {
        ticket,
        user: Box::new(user(false)),
    });
    assert_eq!(
        effects,
        vec![Effect::Toast(Toast::normal(ToastKind::Info, NOT_VERIFIED))]
    );
    assert!(!state.is_checking());

    let ticket = ticket_of(&state.update(Action::CheckVerification));
    let effects = state.update(Action::UserLoaded {
        ticket,
        user: Box::new(user(true)),
    });
    assert_eq!(
        effects,
        vec![Effect::Toast(Toast::normal(
            ToastKind::Success,
            EMAIL_VERIFIED
        ))]
    );
    assert!(state.is_verified());
    assert!(!state.rows().contains(&Row::Verification));
}

#[test]
fn a_failed_check_frees_its_button() {
    let mut state = opened();
    let ticket = ticket_of(&state.update(Action::CheckVerification));
    assert_eq!(state.update(Action::Failed { ticket }), none());
    assert!(!state.is_checking());
}

#[test]
fn resend_holds_the_cooldown_from_the_press_and_runs_it_from_the_reply() {
    let mut state = opened();
    assert_eq!(state.resend_label(), "Resend");
    let effects = state.update(Action::Resend);
    let ticket = ticket_of(&effects);
    assert_eq!(effects, vec![Effect::ResendVerification(ticket)]);
    assert_eq!(state.resend_label(), "Sending...");
    assert_eq!(state.cooldown_secs(), 60);
    assert!(!state.can_resend());
    assert_eq!(
        state.update(Action::Resend),
        none(),
        "a second press can't race the first"
    );

    let effects = state.update(Action::Resent { ticket });
    let wait = wait_of(&effects);
    assert_eq!(
        effects,
        vec![
            Effect::Toast(Toast::normal(ToastKind::Success, VERIFICATION_SENT)),
            Effect::Wait(wait),
        ]
    );
    assert_eq!(state.resend_label(), "Resend in 60s");

    let _ = state.update(Action::Tick { ticket: wait });
    assert_eq!(state.resend_label(), "Resend in 59s");
    assert_eq!(run_down(&mut state, wait), 59);
    assert_eq!(state.cooldown_secs(), 0);
    assert!(state.can_resend());
    assert_eq!(state.resend_label(), "Resend");
}

#[test]
fn a_limited_resend_keeps_the_cooldown_with_an_info_toast() {
    let mut state = opened();
    let ticket = ticket_of(&state.update(Action::Resend));
    let effects = state.update(Action::ResendLimited { ticket });
    let wait = wait_of(&effects);
    assert_eq!(
        effects,
        vec![
            Effect::Toast(Toast::normal(ToastKind::Info, RESEND_LIMITED)),
            Effect::Wait(wait),
        ]
    );
    assert_eq!(state.cooldown_secs(), 60);
    assert!(!state.can_resend());
}

#[test]
fn a_failed_resend_clears_the_cooldown() {
    let mut state = opened();
    let ticket = ticket_of(&state.update(Action::Resend));
    assert_eq!(state.update(Action::Failed { ticket }), none());
    assert_eq!(state.cooldown_secs(), 0);
    assert!(state.can_resend());
    assert_eq!(state.resend_label(), "Resend");
}

#[test]
fn a_late_resend_reply_is_ignored() {
    let mut state = opened();
    let old = ticket_of(&state.update(Action::Resend));
    let _ = state.update(Action::Load(Some(Box::new(user(false)))));
    assert_eq!(state.update(Action::Resent { ticket: old }), none());
    assert_eq!(state.cooldown_secs(), 0);
}

#[test]
fn a_reply_of_the_wrong_kind_leaves_its_request_waiting() {
    let mut state = opened();
    let ticket = ticket_of(&state.update(Action::Resend));
    assert_eq!(state.update(Action::Deleted { ticket }), none());
    assert_eq!(state.update(Action::PasswordChanged { ticket }), none());
    assert!(state.is_sending(), "the resend still waits");
}

#[test]
fn a_stray_tick_changes_nothing() {
    let mut state = opened();
    let ticket = ticket_of(&state.update(Action::Resend));
    assert_eq!(state.update(Action::Tick { ticket }), none());
    assert_eq!(state.cooldown_secs(), 60);
}

#[test]
fn the_username_sheet_judges_each_field_once_it_has_held_something() {
    let mut state = opened();
    assert_eq!(
        state.update(Action::OpenSheet(Sheet::Username)),
        vec![Effect::OpenSheet(Sheet::Username)]
    );
    assert_eq!(state.error(Sheet::Username, Field::NewUsername), None);
    edit(&mut state, Sheet::Username, Field::NewUsername, "ab");
    assert_eq!(
        state.error(Sheet::Username, Field::NewUsername).as_deref(),
        Some("Must be at least 3 characters long")
    );
    assert_eq!(
        state.error(Sheet::Username, Field::Password),
        None,
        "an untouched field waits for Save"
    );
    edit(&mut state, Sheet::Username, Field::NewUsername, "");
    assert!(
        state.error(Sheet::Username, Field::NewUsername).is_some(),
        "an emptied field stays judged"
    );
}

#[test]
fn save_with_a_bad_field_shows_every_line_and_sends_nothing() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Email));
    edit(&mut state, Sheet::Email, Field::NewEmail, "me.zwipe.net");
    assert_eq!(state.update(Action::Save(Sheet::Email)), none());
    assert_eq!(
        state.error(Sheet::Email, Field::NewEmail).as_deref(),
        Some("Missing @ symbol")
    );
    assert_eq!(
        state.error(Sheet::Email, Field::Password).as_deref(),
        Some(account::PASSWORD_REQUIRED)
    );
    assert!(!state.is_saving(Sheet::Email));
}

#[test]
fn a_username_change_stores_the_name_toasts_and_closes() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Username));
    edit(&mut state, Sheet::Username, Field::NewUsername, "newname");
    edit(&mut state, Sheet::Username, Field::Password, "hunter22");
    let effects = state.update(Action::Save(Sheet::Username));
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        vec![Effect::ChangeUsername {
            ticket,
            username: "newname".to_owned(),
            password: Password::new("hunter22"),
        }]
    );
    assert!(state.is_saving(Sheet::Username));
    assert_eq!(state.save_label(Sheet::Username), SAVING);
    assert_eq!(
        state.update(Action::Save(Sheet::Username)),
        none(),
        "Save is off while the change is out"
    );

    let effects = state.update(Action::UsernameChanged {
        ticket,
        username: Username::new("newname").unwrap(),
    });
    assert_eq!(
        effects,
        vec![
            Effect::Toast(Toast::quick(
                ToastKind::Success,
                "Username changed to newname"
            )),
            Effect::CloseSheet(Sheet::Username),
        ]
    );
    assert_eq!(
        state.user().map(|u| u.username.to_string()).as_deref(),
        Some("newname")
    );
    assert_eq!(state.value(Sheet::Username, Field::NewUsername), "");
    assert_eq!(state.save_label(Sheet::Username), SAVE);
}

#[test]
fn an_email_change_takes_the_whole_user_and_names_the_typed_address() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Email));
    edit(&mut state, Sheet::Email, Field::NewEmail, "new@zwipe.net");
    edit(&mut state, Sheet::Email, Field::Password, "hunter22");
    let ticket = ticket_of(&state.update(Action::Save(Sheet::Email)));

    let mut back = user(false);
    back.pending_email = Some(Email::new("new@zwipe.net").unwrap());
    let effects = state.update(Action::EmailChanged {
        ticket,
        user: Box::new(back),
    });
    assert_eq!(
        effects,
        vec![
            Effect::Toast(Toast::normal(
                ToastKind::Success,
                "Check new@zwipe.net for a link to confirm the change"
            )),
            Effect::CloseSheet(Sheet::Email),
        ]
    );
    assert!(state.rows().contains(&Row::Pending));
}

#[test]
fn a_password_change_confirms_first_then_sends() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Password));
    edit(
        &mut state,
        Sheet::Password,
        Field::CurrentPassword,
        "Old!pass1",
    );
    edit(
        &mut state,
        Sheet::Password,
        Field::NewPassword,
        "New!pass22",
    );
    assert_eq!(
        state.error(Sheet::Password, Field::NewPassword),
        None,
        "the confirmation is still being typed"
    );
    edit(
        &mut state,
        Sheet::Password,
        Field::ConfirmPassword,
        "New!pass22",
    );
    assert_eq!(
        state.update(Action::Save(Sheet::Password)),
        vec![Effect::OpenDialog(Dialog::ConfirmPassword)]
    );
    let effects = state.update(Action::ConfirmPassword);
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        vec![Effect::ChangePassword {
            ticket,
            current: Password::new("Old!pass1"),
            new: Password::new("New!pass22"),
        }]
    );
    assert_eq!(
        state.update(Action::PasswordChanged { ticket }),
        vec![
            Effect::Toast(Toast::quick(ToastKind::Success, PASSWORD_CHANGED)),
            Effect::CloseSheet(Sheet::Password),
        ]
    );
}

#[test]
fn a_password_that_repeats_the_current_one_is_refused() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Password));
    for field in Sheet::Password.fields() {
        edit(&mut state, Sheet::Password, *field, "Same!pass1");
    }
    assert_eq!(state.update(Action::Save(Sheet::Password)), none());
    assert_eq!(
        state.error(Sheet::Password, Field::NewPassword).as_deref(),
        Some(account::PASSWORD_UNCHANGED)
    );
    assert_eq!(state.error(Sheet::Password, Field::CurrentPassword), None);
    assert_eq!(state.error(Sheet::Password, Field::ConfirmPassword), None);
}

#[test]
fn a_username_or_email_that_repeats_the_current_one_is_refused() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Username));
    edit(&mut state, Sheet::Username, Field::NewUsername, "scotty");
    edit(&mut state, Sheet::Username, Field::Password, "hunter22");
    assert_eq!(state.update(Action::Save(Sheet::Username)), none());
    assert_eq!(
        state.error(Sheet::Username, Field::NewUsername).as_deref(),
        Some(account::USERNAME_UNCHANGED)
    );

    let _ = state.update(Action::OpenSheet(Sheet::Email));
    edit(
        &mut state,
        Sheet::Email,
        Field::NewEmail,
        "scotty@zwipe.net",
    );
    edit(&mut state, Sheet::Email, Field::Password, "hunter22");
    assert_eq!(state.update(Action::Save(Sheet::Email)), none());
    assert_eq!(
        state.error(Sheet::Email, Field::NewEmail).as_deref(),
        Some(account::EMAIL_UNCHANGED)
    );
}

#[test]
fn a_username_that_only_changes_case_is_sent() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Username));
    edit(&mut state, Sheet::Username, Field::NewUsername, "Scotty");
    edit(&mut state, Sheet::Username, Field::Password, "hunter22");
    assert_eq!(state.error(Sheet::Username, Field::NewUsername), None);
    assert!(
        state
            .update(Action::Save(Sheet::Username))
            .iter()
            .any(|effect| matches!(effect, Effect::ChangeUsername { .. }))
    );
}

#[test]
fn a_failed_change_keeps_the_sheet_as_typed() {
    let mut state = opened();
    let _ = state.update(Action::OpenSheet(Sheet::Username));
    edit(&mut state, Sheet::Username, Field::NewUsername, "newname");
    edit(&mut state, Sheet::Username, Field::Password, "hunter22");
    let ticket = ticket_of(&state.update(Action::Save(Sheet::Username)));
    assert_eq!(state.update(Action::Failed { ticket }), none());
    assert!(!state.is_saving(Sheet::Username));
    assert_eq!(state.value(Sheet::Username, Field::NewUsername), "newname");
}

#[test]
fn opening_a_sheet_empties_only_that_sheet() {
    let mut state = opened();
    edit(&mut state, Sheet::Username, Field::NewUsername, "ab");
    edit(&mut state, Sheet::Email, Field::NewEmail, "x");
    let _ = state.update(Action::OpenSheet(Sheet::Username));
    assert_eq!(state.value(Sheet::Username, Field::NewUsername), "");
    assert_eq!(state.error(Sheet::Username, Field::NewUsername), None);
    assert_eq!(state.value(Sheet::Email, Field::NewEmail), "x");
}

#[test]
fn a_field_not_on_the_sheet_is_ignored() {
    let mut state = opened();
    edit(&mut state, Sheet::Username, Field::NewEmail, "x");
    assert_eq!(state.value(Sheet::Username, Field::NewEmail), "");
}

#[test]
fn a_late_change_reply_is_ignored() {
    let mut state = opened();
    edit(&mut state, Sheet::Username, Field::NewUsername, "newname");
    edit(&mut state, Sheet::Username, Field::Password, "hunter22");
    let ticket = ticket_of(&state.update(Action::Save(Sheet::Username)));
    let _ = state.update(Action::Load(Some(Box::new(user(false)))));
    assert_eq!(
        state.update(Action::UsernameChanged {
            ticket,
            username: Username::new("newname").unwrap(),
        }),
        none()
    );
    assert_eq!(
        state.user().map(|u| u.username.to_string()).as_deref(),
        Some("scotty")
    );
}

#[test]
fn delete_counts_down_five_seconds_before_it_takes_a_press() {
    let mut state = opened();
    let effects = state.update(Action::More(MoreItem::DeleteAccount));
    let wait = wait_of(&effects);
    assert_eq!(
        effects,
        vec![
            Effect::OpenDialog(Dialog::DeleteAccount),
            Effect::Wait(wait)
        ]
    );
    assert_eq!(state.delete_label(), "Delete (5)");
    assert!(!state.can_delete());
    assert_eq!(
        state.update(Action::Delete {
            password: Password::new("hunter22")
        }),
        none(),
        "Delete is off through the countdown"
    );
    assert_eq!(run_down(&mut state, wait), 5);
    assert_eq!(state.delete_label(), "Delete");
    assert!(state.can_delete());

    let effects = state.update(Action::Delete {
        password: Password::new("hunter22"),
    });
    let ticket = ticket_of(&effects);
    assert_eq!(
        effects,
        vec![Effect::DeleteAccount {
            ticket,
            password: Password::new("hunter22"),
        }]
    );
    assert!(state.is_deleting());
    assert!(!state.can_delete());
    assert_eq!(
        state.update(Action::Deleted { ticket }),
        vec![Effect::LogOut]
    );
}

#[test]
fn a_failed_delete_leaves_the_dialog_ready_to_try_again() {
    let mut state = opened();
    let wait = wait_of(&state.update(Action::More(MoreItem::DeleteAccount)));
    run_down(&mut state, wait);
    let ticket = ticket_of(&state.update(Action::Delete {
        password: Password::new("wrong"),
    }));
    assert_eq!(state.update(Action::Failed { ticket }), none());
    assert!(state.can_delete());
}

#[test]
fn reopening_delete_restarts_the_count_without_the_old_waits() {
    let mut state = opened();
    let old = wait_of(&state.update(Action::More(MoreItem::DeleteAccount)));
    let _ = state.update(Action::Tick { ticket: old });
    let _ = state.update(Action::Tick { ticket: old });
    assert_eq!(state.delete_secs(), 3);
    assert_eq!(state.update(Action::CancelDelete), none());
    let new = wait_of(&state.update(Action::More(MoreItem::DeleteAccount)));
    assert_eq!(state.delete_secs(), 5);
    assert_eq!(
        state.update(Action::Tick { ticket: old }),
        none(),
        "the old count's wait"
    );
    assert_eq!(state.delete_secs(), 5);
    assert_eq!(run_down(&mut state, new), 5);
}

#[test]
fn closing_delete_stops_the_count() {
    let mut state = opened();
    let wait = wait_of(&state.update(Action::More(MoreItem::DeleteAccount)));
    let _ = state.update(Action::CancelDelete);
    assert_eq!(state.update(Action::Tick { ticket: wait }), none());
    assert_eq!(state.delete_secs(), 0);
    assert!(!state.can_delete(), "a closed dialog takes no press");
}

#[test]
fn the_resend_and_delete_counts_run_side_by_side() {
    let mut state = opened();
    let ticket = ticket_of(&state.update(Action::Resend));
    let resend = wait_of(&state.update(Action::Resent { ticket }));
    let delete = wait_of(&state.update(Action::More(MoreItem::DeleteAccount)));
    let _ = state.update(Action::Tick { ticket: delete });
    assert_eq!(state.delete_secs(), 4);
    assert_eq!(state.cooldown_secs(), 60);
    let _ = state.update(Action::Tick { ticket: resend });
    assert_eq!(state.cooldown_secs(), 59);
    assert_eq!(state.delete_secs(), 4);
}

#[test]
fn navigation_and_logging_out() {
    let mut state = opened();
    assert_eq!(
        state.update(Action::Back),
        vec![Effect::Navigate(Destination::Previous)]
    );
    assert_eq!(
        state.update(Action::PrivacyPolicy),
        vec![Effect::Navigate(Destination::PrivacyPolicy)]
    );
    assert_eq!(
        state.update(Action::Changelog),
        vec![Effect::Navigate(Destination::Changelog)]
    );
    assert_eq!(
        state.update(Action::Website),
        vec![Effect::OpenUrl(WEB_BASE.to_owned())]
    );
    assert_eq!(
        state.update(Action::LogOut),
        vec![Effect::OpenDialog(Dialog::LogOut)]
    );
    assert_eq!(state.update(Action::ConfirmLogOut), vec![Effect::LogOut]);
}

#[test]
fn passwords_stay_out_of_debug_output() {
    let mut state = opened();
    edit(&mut state, Sheet::Username, Field::NewUsername, "newname");
    edit(&mut state, Sheet::Username, Field::Password, "hunter22");
    let effects = state.update(Action::Save(Sheet::Username));
    let printed = format!("{state:?} {effects:?}");
    assert!(!printed.contains("hunter22"), "{printed}");
    assert!(printed.contains("newname"));
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Account");
    assert_eq!((ACCOUNT, ABOUT), ("Account", "About"));
    assert_eq!(CHANGE, "Change");
    assert_eq!((VERIFIED, UNVERIFIED), ("Verified", "Unverified"));
    assert_eq!(PASSWORD_MASK, "•••••••");
    assert_eq!(WEBSITE_LINK, "zwipe.net ↗");
    assert_eq!((VIEW, CHANGELOG), ("View", "Changelog"));
    assert_eq!((BACK, LOG_OUT, MORE), ("Back", "Log out", "More"));
    assert_eq!(MORE_TITLE, "More");
    assert_eq!(MoreItem::DeleteAccount.label(), "Delete account");
    assert_eq!(CANCEL, "Cancel");
    assert_eq!(DELETE_PASSWORD, "Password");
    assert_eq!(DELETING, "Deleting...");
    assert_eq!((SAVE, SAVING), ("Save changes", "Saving..."));
    assert_eq!(version_label("1.15.0"), "v1.15.0");
    let labels: Vec<&str> = [
        Row::Username,
        Row::Email,
        Row::Pending,
        Row::Verification,
        Row::Password,
        Row::Website,
        Row::PrivacyPolicy,
        Row::Version,
    ]
    .into_iter()
    .map(Row::label)
    .collect();
    assert_eq!(
        labels,
        [
            "Username",
            "Email",
            "Pending",
            "Verification",
            "Password",
            "Website",
            "Privacy policy",
            "Version"
        ]
    );
    let titles: Vec<&str> = Sheet::ALL.into_iter().map(Sheet::title).collect();
    assert_eq!(
        titles,
        ["Change username", "Change email", "Change password"]
    );
    let fields: Vec<(&str, &str, bool)> = Sheet::ALL
        .iter()
        .flat_map(|sheet| sheet.fields())
        .map(|f| (f.label(), f.placeholder(), f.is_masked()))
        .collect();
    assert_eq!(
        fields,
        [
            ("New username", "New username", false),
            ("Password", "Password", true),
            ("New email", "New email", false),
            ("Password", "Password", true),
            ("Current password", "Current password", true),
            ("New password", "New password", true),
            ("Confirm password", "Confirm new", true),
        ]
    );
    assert_eq!(
        (
            Dialog::ConfirmPassword.title(),
            Dialog::ConfirmPassword.body(),
            Dialog::ConfirmPassword.confirm()
        ),
        (
            "Change password",
            "Changing your password will log you out on all other devices.",
            "Confirm"
        )
    );
    assert_eq!(
        (
            Dialog::LogOut.title(),
            Dialog::LogOut.body(),
            Dialog::LogOut.confirm()
        ),
        ("Log out", "Are you sure you want to log out?", "Log out")
    );
    assert_eq!(
        (
            Dialog::DeleteAccount.title(),
            Dialog::DeleteAccount.body(),
            Dialog::DeleteAccount.confirm()
        ),
        (
            "Delete account",
            "This will permanently delete your account, all decks, and all card data. This cannot be undone.",
            "Delete"
        )
    );
    assert_eq!(VERIFICATION_SENT, "Verification email sent");
    assert_eq!(RESEND_LIMITED, "Please wait a moment");
    assert_eq!(EMAIL_VERIFIED, "Email verified");
    assert_eq!(NOT_VERIFIED, "Not verified yet. Check your inbox");
    assert_eq!(PASSWORD_CHANGED, "Password change successful");
}

#[test]
fn the_waits_are_the_app_s() {
    assert_eq!(TICK, Duration::from_secs(1));
    assert_eq!(timing::RESEND_COOLDOWN, Duration::from_secs(60));
    assert_eq!(timing::DELETE_COUNTDOWN, Duration::from_secs(5));
}

#[test]
fn an_edit_keeps_what_was_typed_out_of_debug() {
    let action = Action::Edit {
        sheet: Sheet::Password,
        field: Field::NewPassword,
        value: Typed::new("New!pass22"),
    };
    let printed = format!("{action:?}");
    assert!(!printed.contains("New!pass22"), "{printed}");
    assert!(printed.contains("Typed(10 chars)"), "{printed}");
}
