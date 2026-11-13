//! Which account a server-side admin command names: by username, id or email.

use std::fmt::{self, Display};
use std::str::FromStr;

use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::user::username::{InvalidUsername, Username};

/// An account named on the command line, read once into the kind of key it is.
///
/// A uuid is an id, a valid email is an email, and anything else is a username. A username has
/// at most 20 characters, so it is never an id; one that is itself a valid email (usernames may
/// hold an `@`) reads as an email, so that account is named by its id.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountRef {
    /// The account's exact username.
    Username(Username),
    /// The account's id.
    Id(Uuid),
    /// The account's exact email.
    Email(Email),
}

/// Text that is not an id, not an email and not a valid username.
#[derive(Debug, Clone, thiserror::Error)]
#[error("not an id, an email or a username: {0}")]
pub struct ParseAccountRefError(#[from] InvalidUsername);

impl FromStr for AccountRef {
    type Err = ParseAccountRefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = Uuid::parse_str(s) {
            return Ok(Self::Id(id));
        }
        if let Ok(email) = Email::new(s) {
            return Ok(Self::Email(email));
        }
        Ok(Self::Username(Username::new(s)?))
    }
}

impl Display for AccountRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Username(username) => write!(f, "{username}"),
            Self::Id(id) => write!(f, "{id}"),
            Self::Email(email) => write!(f, "{email}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_of_key_reads_as_itself() {
        let id = Uuid::new_v4();
        assert_eq!(
            id.to_string().parse::<AccountRef>().unwrap(),
            AccountRef::Id(id)
        );
        assert!(matches!(
            "someone@example.com".parse::<AccountRef>(),
            Ok(AccountRef::Email(email)) if email.as_ref() == "someone@example.com"
        ));
        assert!(matches!(
            "Test".parse::<AccountRef>(),
            Ok(AccountRef::Username(name)) if &*name == "Test"
        ));
    }

    #[test]
    fn text_with_an_at_that_is_no_email_reads_as_a_username() {
        assert!(matches!(
            "not@an@email".parse::<AccountRef>(),
            Ok(AccountRef::Username(name)) if &*name == "not@an@email"
        ));
    }

    #[test]
    fn text_that_is_no_key_fails_to_parse() {
        assert!("ab".parse::<AccountRef>().is_err());
    }
}
