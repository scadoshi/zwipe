//! Profile, preferences and the commander maybeboard.

use std::borrow::Cow;

use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::user::User;
use crate::domain::user::models::preferences::UserPreferences;
use crate::http::contracts::auth::{
    HttpChangeEmail, HttpChangePassword, HttpChangeUsername, HttpDeleteUser,
};
use crate::http::contracts::user::{HttpMarkHintShown, HttpUpdatePreferences};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths;
use crate::http::paths::{
    CHANGE_EMAIL_ROUTE, CHANGE_PASSWORD_ROUTE, CHANGE_USERNAME_ROUTE,
    CLEAR_COMMANDER_MAYBEBOARD_ROUTE, DELETE_USER_ROUTE, GET_COMMANDER_MAYBEBOARD_ROUTE,
    GET_USER_ROUTE, MARK_HINT_SHOWN_ROUTE, PREFERENCES_ROUTE,
};

/// The signed-in user's profile.
#[derive(Debug)]
pub struct GetUser;
impl Endpoint for GetUser {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = User;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(GET_USER_ROUTE)
    }
}

/// Change the account email; answers the updated user.
#[derive(Debug)]
pub struct ChangeEmail(pub HttpChangeEmail);
impl Endpoint for ChangeEmail {
    const METHOD: Method = Method::Patch;
    type Request = HttpChangeEmail;
    type Response = User;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CHANGE_EMAIL_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Change the username; answers the updated user.
#[derive(Debug)]
pub struct ChangeUsername(pub HttpChangeUsername);
impl Endpoint for ChangeUsername {
    const METHOD: Method = Method::Patch;
    type Request = HttpChangeUsername;
    type Response = User;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CHANGE_USERNAME_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Change the password. Answers no content beyond 200.
#[derive(Debug)]
pub struct ChangePassword(pub HttpChangePassword);
impl Endpoint for ChangePassword {
    const METHOD: Method = Method::Patch;
    type Request = HttpChangePassword;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CHANGE_PASSWORD_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Delete the account. Answers 204.
#[derive(Debug)]
pub struct DeleteUser(pub HttpDeleteUser);
impl Endpoint for DeleteUser {
    const METHOD: Method = Method::Delete;
    type Request = HttpDeleteUser;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(DELETE_USER_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Mark a one-time UI hint as seen; answers the updated user.
#[derive(Debug)]
pub struct MarkHintShown(pub HttpMarkHintShown);
impl Endpoint for MarkHintShown {
    const METHOD: Method = Method::Patch;
    type Request = HttpMarkHintShown;
    type Response = User;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(MARK_HINT_SHOWN_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Read display preferences.
#[derive(Debug)]
pub struct GetPreferences;
impl Endpoint for GetPreferences {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = UserPreferences;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(PREFERENCES_ROUTE)
    }
}

/// Update display preferences; answers the stored result.
#[derive(Debug)]
pub struct UpdatePreferences(pub HttpUpdatePreferences);
impl Endpoint for UpdatePreferences {
    const METHOD: Method = Method::Patch;
    type Request = HttpUpdatePreferences;
    type Response = UserPreferences;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(PREFERENCES_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// The saved commander candidates.
#[derive(Debug)]
pub struct GetCommanderMaybeboard;
impl Endpoint for GetCommanderMaybeboard {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = Vec<Card>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(GET_COMMANDER_MAYBEBOARD_ROUTE)
    }
}

/// Save one commander candidate. Answers 204.
#[derive(Debug)]
pub struct AddCommanderMaybeboardCard(pub Uuid);
impl Endpoint for AddCommanderMaybeboardCard {
    const METHOD: Method = Method::Post;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(paths::add_commander_maybeboard_card_route(self.0))
    }
}

/// Drop one commander candidate. Answers 204.
#[derive(Debug)]
pub struct RemoveCommanderMaybeboardCard(pub Uuid);
impl Endpoint for RemoveCommanderMaybeboardCard {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(paths::remove_commander_maybeboard_card_route(self.0))
    }
}

/// Empty the commander maybeboard. Answers 204.
#[derive(Debug)]
pub struct ClearCommanderMaybeboard;
impl Endpoint for ClearCommanderMaybeboard {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CLEAR_COMMANDER_MAYBEBOARD_ROUTE)
    }
}
