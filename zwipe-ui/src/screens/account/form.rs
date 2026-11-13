//! The Account screen's three change sheets: their fields, the words on them, and what each field shows under it as the user types and saves.

use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};

use zwipe_core::domain::user::User;

use crate::account;

/// The Save button's label while a change is out.
pub const SAVING: &str = "Saving…";

/// The Save button's label.
pub const SAVE: &str = "Save changes";

/// A change sheet over the Account screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sheet {
    /// Change username: a new username and the password.
    Username,
    /// Change email: a new address and the password.
    Email,
    /// Change password: the current password, a new one and its confirmation. Save asks [`Dialog::ConfirmPassword`](super::Dialog::ConfirmPassword) first.
    Password,
}

impl Sheet {
    /// Every sheet, in the order the screen's rows offer them.
    pub const ALL: [Self; 3] = [Self::Username, Self::Email, Self::Password];

    /// The sheet's title.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Username => "Change username",
            Self::Email => "Change email",
            Self::Password => "Change password",
        }
    }

    /// The sheet's fields, top to bottom.
    #[must_use]
    pub fn fields(self) -> &'static [Field] {
        match self {
            Self::Username => &[Field::NewUsername, Field::Password],
            Self::Email => &[Field::NewEmail, Field::Password],
            Self::Password => &[
                Field::CurrentPassword,
                Field::NewPassword,
                Field::ConfirmPassword,
            ],
        }
    }
}

/// A field on a change sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Field {
    /// The username to switch to.
    NewUsername,
    /// The address to switch to.
    NewEmail,
    /// The account's password, on Change username and Change email.
    Password,
    /// The password in use, on Change password. Never judged here, so a password that predates the policy can still be changed.
    CurrentPassword,
    /// The password to switch to. Its line covers the confirmation too.
    NewPassword,
    /// The new password again.
    ConfirmPassword,
}

impl Field {
    /// Whether this field holds a password. The password policy is the server's, so a password field's error line never holds Save.
    #[must_use]
    pub fn is_password(self) -> bool {
        matches!(
            self,
            Self::Password | Self::CurrentPassword | Self::NewPassword | Self::ConfirmPassword
        )
    }

    /// The label above the field.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::NewUsername => "New username",
            Self::NewEmail => "New email",
            Self::Password => "Password",
            Self::CurrentPassword => "Current password",
            Self::NewPassword => "New password",
            Self::ConfirmPassword => "Confirm password",
        }
    }

    /// Whether the field hides what is typed.
    #[must_use]
    pub fn is_masked(self) -> bool {
        matches!(
            self,
            Self::Password | Self::CurrentPassword | Self::NewPassword | Self::ConfirmPassword
        )
    }
}

/// One sheet's fields as typed, and whether a save was tried.
#[derive(Default, Clone, PartialEq, Eq)]
pub(super) struct Form {
    values: BTreeMap<Field, String>,
    attempted: bool,
}

// By hand: the derive would print the passwords.
impl Debug for Form {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let values: BTreeMap<Field, &str> = self
            .values
            .iter()
            .map(|(field, value)| {
                let shown = if field.is_masked() {
                    "REDACTED"
                } else {
                    value.as_str()
                };
                (*field, shown)
            })
            .collect();
        f.debug_struct("Form")
            .field("values", &values)
            .field("attempted", &self.attempted)
            .finish()
    }
}

impl Form {
    /// What `field` holds.
    pub(super) fn value(&self, field: Field) -> &str {
        self.values.get(&field).map_or("", String::as_str)
    }

    /// Sets `field` to `value`.
    pub(super) fn edit(&mut self, field: Field, value: String) {
        self.values.insert(field, value);
    }

    /// Marks a save tried, so every field is judged.
    pub(super) fn attempt(&mut self) {
        self.attempted = true;
    }

    /// The line under `field` on `sheet`, while the field holds text or once a save was tried; an emptied field shows none until then. Change password's one line sits under the new password and covers the confirmation. `current` is the signed-in user, whose username and email a change may not repeat.
    pub(super) fn error(
        &self,
        sheet: Sheet,
        field: Field,
        current: Option<&User>,
    ) -> Option<String> {
        let judged = |f: Field| self.attempted || !self.value(f).is_empty();
        match (sheet, field) {
            (Sheet::Username, Field::NewUsername) if judged(field) => match current {
                Some(user) => {
                    account::change_username_error(user.username.as_ref(), self.value(field))
                }
                None => account::username_error(self.value(field)),
            },
            (Sheet::Email, Field::NewEmail) if judged(field) => match current {
                Some(user) => account::change_email_error(user.email.as_ref(), self.value(field)),
                None => account::email_error(self.value(field)),
            },
            (Sheet::Username | Sheet::Email, Field::Password) if judged(field) => {
                account::password_required(self.value(field))
            }
            (Sheet::Password, Field::NewPassword)
                if judged(Field::NewPassword) || judged(Field::ConfirmPassword) =>
            {
                account::change_password_error(
                    self.value(Field::CurrentPassword),
                    self.value(Field::NewPassword),
                    self.value(Field::ConfirmPassword),
                    self.attempted,
                )
            }
            _ => None,
        }
    }

    /// Whether no field on `sheet` has a line under it.
    pub(super) fn passes(&self, sheet: Sheet, current: Option<&User>) -> bool {
        sheet
            .fields()
            .iter()
            .all(|field| self.error(sheet, *field, current).is_none())
    }
}

/// A field's whole text as typed, carried by [`Action::Edit`](super::Action::Edit). Its `Debug` prints only the length, since the field may hold a password.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Typed(String);

impl Typed {
    /// Wraps a field's text as typed.
    #[must_use]
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// The text as typed.
    #[must_use]
    pub fn read(&self) -> &str {
        &self.0
    }

    /// The text as typed, for the form to keep.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

// By hand: the derive would print the text, which may be a password.
impl Debug for Typed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Typed({} chars)", self.0.chars().count())
    }
}
