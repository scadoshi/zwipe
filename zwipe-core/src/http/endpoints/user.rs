//! Profile, preferences and the commander maybeboard.

use crate::{
    domain::{
        card::Card,
        user::{User, models::preferences::UserPreferences},
    },
    http::{
        endpoint::{Endpoint, Method},
        paths::{
            CHANGE_EMAIL_ROUTE, CHANGE_PASSWORD_ROUTE, CHANGE_USERNAME_ROUTE,
            CLEAR_COMMANDER_MAYBEBOARD_ROUTE, DELETE_USER_ROUTE, GET_COMMANDER_MAYBEBOARD_ROUTE,
            GET_USER_ROUTE, MARK_HINT_SHOWN_ROUTE, PREFERENCES_ROUTE,
            add_commander_maybeboard_card_route, remove_commander_maybeboard_card_route,
        },
    },
};
use serde_json::Value;
use uuid::Uuid;

/// The signed-in user's profile.
pub struct GetUser;
impl Endpoint for GetUser {
    const METHOD: Method = Method::Get;
    type Response = User;
    fn path(&self) -> String {
        GET_USER_ROUTE.to_string()
    }
}

/// Change the account email; answers the updated user.
pub struct ChangeEmail(pub Value);
impl Endpoint for ChangeEmail {
    const METHOD: Method = Method::Patch;
    type Response = User;
    fn path(&self) -> String {
        CHANGE_EMAIL_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Change the username; answers the updated user.
pub struct ChangeUsername(pub Value);
impl Endpoint for ChangeUsername {
    const METHOD: Method = Method::Patch;
    type Response = User;
    fn path(&self) -> String {
        CHANGE_USERNAME_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Change the password. Answers no content beyond 200.
pub struct ChangePassword(pub Value);
impl Endpoint for ChangePassword {
    const METHOD: Method = Method::Patch;
    type Response = ();
    fn path(&self) -> String {
        CHANGE_PASSWORD_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Delete the account. Answers 204.
pub struct DeleteUser(pub Value);
impl Endpoint for DeleteUser {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        DELETE_USER_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Mark a one-time UI hint as seen; answers the updated user.
pub struct MarkHintShown(pub Value);
impl Endpoint for MarkHintShown {
    const METHOD: Method = Method::Patch;
    type Response = User;
    fn path(&self) -> String {
        MARK_HINT_SHOWN_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// Read display preferences.
pub struct GetPreferences;
impl Endpoint for GetPreferences {
    const METHOD: Method = Method::Get;
    type Response = UserPreferences;
    fn path(&self) -> String {
        PREFERENCES_ROUTE.to_string()
    }
}

/// Update display preferences; answers the stored result.
pub struct UpdatePreferences(pub Value);
impl Endpoint for UpdatePreferences {
    const METHOD: Method = Method::Patch;
    type Response = UserPreferences;
    fn path(&self) -> String {
        PREFERENCES_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// The saved commander candidates.
pub struct GetCommanderMaybeboard;
impl Endpoint for GetCommanderMaybeboard {
    const METHOD: Method = Method::Get;
    type Response = Vec<Card>;
    fn path(&self) -> String {
        GET_COMMANDER_MAYBEBOARD_ROUTE.to_string()
    }
}

/// Save one commander candidate. Answers 204.
pub struct AddCommanderMaybeboardCard(pub Uuid);
impl Endpoint for AddCommanderMaybeboardCard {
    const METHOD: Method = Method::Post;
    type Response = ();
    fn path(&self) -> String {
        add_commander_maybeboard_card_route(self.0)
    }
}

/// Drop one commander candidate. Answers 204.
pub struct RemoveCommanderMaybeboardCard(pub Uuid);
impl Endpoint for RemoveCommanderMaybeboardCard {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        remove_commander_maybeboard_card_route(self.0)
    }
}

/// Empty the commander maybeboard. Answers 204.
pub struct ClearCommanderMaybeboard;
impl Endpoint for ClearCommanderMaybeboard {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        CLEAR_COMMANDER_MAYBEBOARD_ROUTE.to_string()
    }
}
