//! The account screen's state: the user, the selected row, and the open form or dialog.

use std::time::{Duration, Instant};

use zwipe_core::domain::user::User;

use crate::domain::account::{
    self, DELETE_DELAY_SECS, RESEND_COOLDOWN_SECS, email_error, new_password_error,
    password_required, username_error,
};

/// The account screen.
#[derive(Debug)]
pub(crate) struct AccountView {
    pub(crate) user: User,
    /// The selected row of [`AccountView::rows`].
    pub(crate) row: usize,
    pub(crate) dialog: Option<Dialog>,
    /// When the verification email can be sent again.
    pub(crate) resend_ready_at: Option<Instant>,
}

/// A row that does something when picked, in the app's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccountRow {
    Username,
    Email,
    /// Sends the verification email again. Only while the email is unverified.
    Resend,
    /// Asks the server whether the email has been verified. Only while it is unverified.
    Check,
    Password,
    LogOut,
    Delete,
}

impl AccountView {
    pub(crate) fn new(user: User) -> Self {
        Self {
            user,
            row: 0,
            dialog: None,
            resend_ready_at: None,
        }
    }

    pub(crate) fn verified(&self) -> bool {
        self.user.email_verified_at.is_some()
    }

    /// The rows that apply to this user.
    pub(crate) fn rows(&self) -> Vec<AccountRow> {
        let mut rows = vec![AccountRow::Username, AccountRow::Email];
        if !self.verified() {
            rows.extend([AccountRow::Resend, AccountRow::Check]);
        }
        rows.extend([AccountRow::Password, AccountRow::LogOut, AccountRow::Delete]);
        rows
    }

    pub(crate) fn selected(&self) -> Option<AccountRow> {
        self.rows().get(self.row).copied()
    }

    pub(crate) fn next_row(&mut self) {
        self.row = self
            .row
            .saturating_add(1)
            .min(self.rows().len().saturating_sub(1));
    }

    pub(crate) fn previous_row(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    /// Takes a fresh copy of the user, keeping the selection in range.
    pub(crate) fn set_user(&mut self, user: User) {
        self.user = user;
        self.row = self.row.min(self.rows().len().saturating_sub(1));
    }

    /// Seconds until the verification email can be sent again, `None` when it can be now.
    pub(crate) fn resend_wait(&self) -> Option<u64> {
        let left = self
            .resend_ready_at?
            .saturating_duration_since(Instant::now());
        (!left.is_zero()).then(|| left.as_secs().saturating_add(1))
    }

    /// Starts the wait between verification emails.
    pub(crate) fn start_resend_cooldown(&mut self) {
        self.resend_ready_at =
            Instant::now().checked_add(Duration::from_secs(RESEND_COOLDOWN_SECS));
    }
}

/// What is open over the account screen.
#[derive(Debug)]
pub(crate) enum Dialog {
    Form(Form),
    /// Asks before changing the password, since that signs out every other device.
    ConfirmPassword(Form),
    LogOut,
}

/// Which form is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormKind {
    Username,
    Email,
    Password,
    Delete,
}

impl FormKind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Username => "Change username",
            Self::Email => "Change email",
            Self::Password => "Change password",
            Self::Delete => "Delete account",
        }
    }

    /// Each field's label, and whether it hides what is typed.
    pub(crate) fn fields(self) -> &'static [(&'static str, bool)] {
        match self {
            Self::Username => &[("New username", false), ("Password", true)],
            Self::Email => &[("New email", false), ("Password", true)],
            Self::Password => &[
                ("Current password", true),
                ("New password", true),
                ("Confirm password", true),
            ],
            Self::Delete => &[("Password", true)],
        }
    }
}

/// An open form: what is typed in each field, which has the cursor, and why the last submit failed.
#[derive(Debug)]
pub(crate) struct Form {
    pub(crate) kind: FormKind,
    pub(crate) values: Vec<String>,
    pub(crate) focus: usize,
    pub(crate) error: Option<String>,
    /// Set while the request is out.
    pub(crate) busy: bool,
    pub(crate) opened_at: Instant,
}

impl Form {
    pub(crate) fn new(kind: FormKind) -> Self {
        Self {
            kind,
            values: vec![String::new(); kind.fields().len()],
            focus: 0,
            error: None,
            busy: false,
            opened_at: Instant::now(),
        }
    }

    pub(crate) fn value(&self, field: usize) -> &str {
        self.values.get(field).map_or("", String::as_str)
    }

    pub(crate) fn focused_mut(&mut self) -> Option<&mut String> {
        self.values.get_mut(self.focus)
    }

    pub(crate) fn next_field(&mut self) {
        self.focus = self.focus.saturating_add(1) % self.values.len().max(1);
    }

    pub(crate) fn previous_field(&mut self) {
        self.focus = self
            .focus
            .checked_sub(1)
            .unwrap_or_else(|| self.values.len().saturating_sub(1));
    }

    pub(crate) fn on_last_field(&self) -> bool {
        self.focus.saturating_add(1) >= self.values.len()
    }

    /// Seconds before the delete form can submit, `None` once it can.
    pub(crate) fn delete_wait(&self) -> Option<u64> {
        if self.kind != FormKind::Delete {
            return None;
        }
        let left = Duration::from_secs(DELETE_DELAY_SECS).saturating_sub(self.opened_at.elapsed());
        (!left.is_zero()).then(|| left.as_secs().saturating_add(1))
    }

    /// Checks the fields the way the app does before sending, giving the first problem.
    pub(crate) fn problem(&self) -> Option<String> {
        match self.kind {
            FormKind::Username => {
                username_error(self.value(0).trim()).or_else(|| password_required(self.value(1)))
            }
            FormKind::Email => {
                email_error(self.value(0).trim()).or_else(|| password_required(self.value(1)))
            }
            FormKind::Password => new_password_error(self.value(0), self.value(1), self.value(2)),
            FormKind::Delete => None,
        }
    }
}

/// The note under a pending email change, for `user`.
pub(crate) fn pending_note(user: &User) -> Option<String> {
    user.pending_email_expires_at
        .map(|at| account::pending_note(at, chrono::Utc::now()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(verified: bool) -> User {
        let mut user: User = serde_json::from_value(serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "username": "scotty",
            "email": "scotty@zwipe.net",
            "email_verified_at": null,
            "hints_shown": {},
        }))
        .unwrap();
        if verified {
            user.email_verified_at = Some(chrono::Utc::now());
        }
        user
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
}
