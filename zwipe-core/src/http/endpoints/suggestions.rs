//! Suggested commanders and their dismissals.

use std::borrow::Cow;

use uuid::Uuid;

use crate::http::contracts::suggestions::{
    HttpActivity, HttpClearedDismissals, HttpCommanderSuggestions, HttpDismissCommander,
    HttpPlayStyleSummary,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    ACTIVITY_ROUTE, COMMANDER_DISMISSALS_ROUTE, COMMANDER_SUGGESTIONS_ROUTE, PLAYSTYLE_ROUTE,
    undismiss_commander_route,
};

/// The user's deck count and color map, for Home's play
/// style panel.
#[derive(Debug)]
pub struct GetPlayStyleSummary;
impl Endpoint for GetPlayStyleSummary {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpPlayStyleSummary;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLAYSTYLE_ROUTE)
    }
}

/// The user's swiping, searching, rates by role and tag, and hard passes,
/// for the Activity screen.
#[derive(Debug)]
pub struct GetActivity;
impl Endpoint for GetActivity {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpActivity;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(ACTIVITY_ROUTE)
    }
}

/// The user's ranked commander suggestions, at most this many. The server
/// clamps the limit to `MAX_SUGGESTION_LIMIT`.
#[derive(Debug)]
pub struct GetCommanderSuggestions(pub u32);
impl Endpoint for GetCommanderSuggestions {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpCommanderSuggestions;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COMMANDER_SUGGESTIONS_ROUTE)
    }
    fn query(&self) -> Option<String> {
        Some(format!("limit={}", self.0))
    }
}

/// Hide one commander from the suggestions for a week. Answers 204.
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct ClearCommanderDismissals;
impl Endpoint for ClearCommanderDismissals {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = HttpClearedDismissals;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COMMANDER_DISMISSALS_ROUTE)
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

    #[test]
    fn play_style_summary_reads_the_playstyle_route() {
        assert_eq!(GetPlayStyleSummary.path(), "/api/deck/playstyle");
        assert_eq!(GetPlayStyleSummary.query(), None);
    }

    #[test]
    fn activity_reads_under_the_user_route() {
        assert_eq!(GetActivity.path(), "/api/user/activity");
    }
}
