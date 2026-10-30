use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home;
use crate::test_support::{screen_text, user};

fn ask(request: Request) -> screens::Request {
    request.into()
}

/// Home with the user known, then the account screen over it, with nothing left queued.
fn on_account(verified: bool) -> Harness {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(verified));
    harness.open(home::open);
    harness.clear();
    harness.press(KeyCode::Char('a'));
    harness
}

fn account_view(harness: &Harness) -> &AccountView {
    match harness.screen() {
        Screen::Account(view) => view,
        other => panic!("not on the account screen: {other:?}"),
    }
}

#[test]
fn a_opens_the_account_and_refreshes_the_user() {
    let harness = on_account(true);
    assert_eq!(account_view(&harness).user.username.to_string(), "scotty");
    assert_eq!(harness.pending(), [ask(Request::LoadUser)]);
}

#[test]
fn a_bad_username_is_refused_in_the_form_and_a_good_one_is_sent() {
    let mut harness = on_account(true);
    harness.clear();
    harness.press(KeyCode::Enter);
    harness.type_text("ab");
    harness.press(KeyCode::Enter);
    harness.type_text("hunter2");
    harness.press(KeyCode::Enter);
    assert!(matches!(
        &account_view(&harness).dialog,
        Some(Dialog::Form(form)) if form.error.as_deref() == Some("must be at least 3 characters long")
    ));
    assert_eq!(harness.pending(), []);

    harness.press(KeyCode::Tab);
    harness.type_text("cdef");
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [ask(Request::Change(AccountChange::Username {
            username: "abcdef".to_owned(),
            password: Password("hunter2".to_owned()),
        }))]
    );
    assert!(!format!("{:?}", harness.pending()).contains("hunter2"));

    harness.apply(Update::ChangeFailed("invalid password".to_owned()));
    assert!(matches!(
        &account_view(&harness).dialog,
        Some(Dialog::Form(form)) if !form.busy && form.error.as_deref() == Some("invalid password")
    ));

    let mut renamed = user(true);
    renamed.username = zwipe_core::domain::user::Username::new("abcdef").unwrap();
    harness.apply(Update::Changed {
        user: Some(Box::new(renamed)),
        message: "Username changed to abcdef".to_owned(),
    });
    assert!(account_view(&harness).dialog.is_none());
    assert_eq!(harness.app.shared.username.as_deref(), Some("abcdef"));
    assert_eq!(harness.app.shared.status, "Username changed to abcdef");
}

#[test]
fn a_password_change_asks_before_it_signs_out_other_devices() {
    let mut harness = on_account(true);
    harness.clear();
    for _ in 0..2 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    for value in ["Old!pass1", "New!pass22", "New!pass22"] {
        harness.type_text(value);
        harness.press(KeyCode::Enter);
    }
    assert!(matches!(
        account_view(&harness).dialog,
        Some(Dialog::ConfirmPassword(_))
    ));
    assert_eq!(harness.pending(), []);
    harness.press(KeyCode::Char('y'));
    assert!(matches!(
        harness.pending(),
        [screens::Request::Account(Request::Change(
            AccountChange::Password { .. }
        ))]
    ));
}

#[test]
fn delete_waits_five_seconds_and_log_out_asks_first() {
    let mut harness = on_account(true);
    harness.clear();
    for _ in 0..10 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    harness.type_text("pw");
    harness.press(KeyCode::Enter);
    assert!(matches!(&account_view(&harness).dialog, Some(Dialog::Form(form)) if !form.busy));
    assert_eq!(
        harness.pending(),
        [],
        "the delete button is still counting down"
    );
    assert!(harness.app.animating(), "the countdown redraws");

    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('k'));
    harness.press(KeyCode::Enter);
    assert!(matches!(
        account_view(&harness).dialog,
        Some(Dialog::LogOut)
    ));
    harness.press(KeyCode::Char('y'));
    assert_eq!(harness.pending(), [screens::Request::SignOut]);
}

#[test]
fn resend_waits_a_minute_between_emails_unless_it_failed() {
    let mut harness = on_account(false);
    harness.clear();
    for _ in 0..2 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), [ask(Request::Resend)]);
    assert!(account_view(&harness).resend_wait().is_some());

    harness.apply(Update::ResendFailed("offline".to_owned()));
    assert!(account_view(&harness).resend_wait().is_none());
    assert_eq!(harness.app.shared.status, "offline");
}

#[test]
fn the_account_screen_shows_details_and_verification_rows() {
    let harness = Harness::on(Screen::Account(Box::new(AccountView::new(user(false)))));
    let screen = screen_text(&harness.app);
    for expected in [
        "> Username",
        "scotty",
        "scotty@zwipe.net",
        "Unverified",
        "Verification",
        "Resend",
        "Check",
        "•••••••",
        "Version",
        "Log out",
        "Delete account",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn a_form_masks_passwords_and_the_delete_button_counts_down() {
    let mut view = AccountView::new(user(true));
    let mut form = Form::new(FormKind::Delete);
    form.values = vec!["hunter2".to_owned()];
    view.dialog = Some(Dialog::Form(form));
    let harness = Harness::on(Screen::Account(Box::new(view)));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Delete account"));
    assert!(screen.contains("permanently delete"));
    assert!(screen.contains("•••••••"));
    assert!(!screen.contains("hunter2"));
    assert!(screen.contains("Delete (5)"));
}

#[test]
fn verification_rows_show_only_while_unverified() {
    assert_eq!(
        AccountView::new(user(true)).rows(),
        [
            AccountRow::Username,
            AccountRow::Email,
            AccountRow::Password,
            AccountRow::LogOut,
            AccountRow::Delete
        ]
    );
    assert!(
        AccountView::new(user(false))
            .rows()
            .contains(&AccountRow::Resend)
    );
}

#[test]
fn verifying_keeps_the_selection_in_range() {
    let mut view = AccountView::new(user(false));
    for _ in 0..10 {
        view.next_row();
    }
    assert_eq!(view.selected(), Some(AccountRow::Delete));
    view.set_user(user(true));
    assert_eq!(view.selected(), Some(AccountRow::Delete));
}

#[test]
fn forms_check_their_fields_before_sending() {
    let mut form = Form::new(FormKind::Username);
    form.values = vec!["ab".to_owned(), "pw".to_owned()];
    assert_eq!(
        form.problem().as_deref(),
        Some("must be at least 3 characters long")
    );
    form.values = vec!["scotty2".to_owned(), String::new()];
    assert_eq!(form.problem().as_deref(), Some("Password is required"));
    form.values = vec!["scotty2".to_owned(), "pw".to_owned()];
    assert_eq!(form.problem(), None);
}

#[test]
fn the_delete_form_waits_before_it_can_submit() {
    let mut form = Form::new(FormKind::Delete);
    assert_eq!(form.delete_wait(), Some(DELETE_DELAY_SECS));
    form.opened_at = Instant::now() - Duration::from_secs(DELETE_DELAY_SECS);
    assert_eq!(form.delete_wait(), None);
    assert_eq!(Form::new(FormKind::Email).delete_wait(), None);
}

#[test]
fn fields_cycle_and_the_last_one_is_known() {
    let mut form = Form::new(FormKind::Password);
    assert!(!form.on_last_field());
    form.previous_field();
    assert!(form.on_last_field());
    form.next_field();
    assert_eq!(form.focus, 0);
}

#[test]
fn the_resend_cooldown_counts_down_from_a_minute() {
    let mut view = AccountView::new(user(false));
    assert_eq!(view.resend_wait(), None);
    view.start_resend_cooldown();
    assert_eq!(view.resend_wait(), Some(RESEND_COOLDOWN_SECS));
}
