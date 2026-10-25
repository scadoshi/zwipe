//! Suggested commanders and their dismissals.

use std::borrow::Cow;

use uuid::Uuid;

use crate::http::contracts::suggestions::{
    HttpClearedDismissals, HttpCommanderSuggestions, HttpDismissCommander,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    COMMANDER_DISMISSALS_ROUTE, commander_suggestions_route, undismiss_commander_route,
};

/// The user's ranked commander suggestions, at most this many. The server
/// clamps the limit to `MAX_SUGGESTION_LIMIT`.
pub struct GetCommanderSuggestions(pub u32);
impl Endpoint for GetCommanderSuggestions {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpCommanderSuggestions;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(commander_suggestions_route(self.0))
    }
}

/// Hide one commander from the suggestions for a week. Answers 204.
pub struct DismissCommander(pub HttpDismissCommander);
impl Endpoint for DismissCommander {
    const METHOD: Method = Method::Post;
    type Request = HttpDismissCommander;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COMMANDER_DISMISSALS_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Undo one dismissal. Answers 204.
pub struct UndismissCommander(pub Uuid);
impl Endpoint for UndismissCommander {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(undismiss_commander_route(self.0))
    }
}

/// Lift every dismissal; answers how many were lifted.
pub struct ClearCommanderDismissals;
impl Endpoint for ClearCommanderDismissals {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = HttpClearedDismissals;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COMMANDER_DISMISSALS_ROUTE)
    }
}
