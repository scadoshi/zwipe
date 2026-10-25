//! Suggested commanders and their dismissals.

use std::borrow::Cow;

use uuid::Uuid;

use crate::http::contracts::suggestions::{
    HttpClearedDismissals, HttpCommanderSuggestions, HttpDismissCommander,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    CLEAR_COMMANDER_DISMISSALS_ROUTE, DISMISS_COMMANDER_ROUTE, GET_COMMANDER_SUGGESTIONS_ROUTE,
    undismiss_commander_route,
};

/// The ranked suggestions, at most `limit` of them (server default 10, max 100).
pub struct GetCommanderSuggestions(pub u32);
impl Endpoint for GetCommanderSuggestions {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpCommanderSuggestions;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(GET_COMMANDER_SUGGESTIONS_ROUTE)
    }
    fn query(&self) -> Option<String> {
        Some(format!("limit={}", self.0))
    }
}

/// Hide one commander from the suggestions for a week. Answers 204.
pub struct DismissCommander(pub HttpDismissCommander);
impl Endpoint for DismissCommander {
    const METHOD: Method = Method::Post;
    type Request = HttpDismissCommander;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(DISMISS_COMMANDER_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Lift one dismissal. Answers 204.
pub struct UndismissCommander(pub Uuid);
impl Endpoint for UndismissCommander {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(undismiss_commander_route(self.0))
    }
}

/// Lift every dismissal.
pub struct ClearCommanderDismissals;
impl Endpoint for ClearCommanderDismissals {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = HttpClearedDismissals;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CLEAR_COMMANDER_DISMISSALS_ROUTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_carry_the_limit_as_a_query() {
        let endpoint = GetCommanderSuggestions(100);
        assert_eq!(endpoint.path(), "/api/commander/suggestions");
        assert_eq!(endpoint.query().as_deref(), Some("limit=100"));
    }
}
