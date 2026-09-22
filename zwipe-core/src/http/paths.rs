//! Path constants shared between frontend and backend for URL consistency.
//!
//! Fixed paths are `const`; only the ones interpolating an id are functions.
//! Every path is absolute, so both `Url::set_path` and a plain
//! `format!("{base}{path}")` join produce the same URL.

#![allow(missing_docs)]

use crate::http::endpoint::Method;
use uuid::Uuid;

// == health ==

pub const HEALTH_ROUTE: &str = "/health";
pub const SERVER_HEALTH_ROUTE: &str = "/health/server";
pub const DATABASE_HEALTH_ROUTE: &str = "/health/database";

// == auth ==

pub const REGISTER_ROUTE: &str = "/api/auth/register";
pub const LOGIN_ROUTE: &str = "/api/auth/login";
pub const REFRESH_SESSION_ROUTE: &str = "/api/auth/refresh";
pub const LOGOUT_ROUTE: &str = "/api/auth/logout";
pub const VERIFY_EMAIL_ROUTE: &str = "/api/auth/verify-email";
pub const FORGOT_PASSWORD_ROUTE: &str = "/api/auth/forgot-password";
pub const RESET_PASSWORD_ROUTE: &str = "/api/auth/reset-password";
pub const RESEND_VERIFICATION_ROUTE: &str = "/api/auth/resend-verification";

// == user ==

pub const GET_USER_ROUTE: &str = "/api/user";
pub const CHANGE_PASSWORD_ROUTE: &str = "/api/user/change-password";
pub const CHANGE_USERNAME_ROUTE: &str = "/api/user/change-username";
pub const CHANGE_EMAIL_ROUTE: &str = "/api/user/change-email";
pub const DELETE_USER_ROUTE: &str = "/api/user/delete-user";
pub const PREFERENCES_ROUTE: &str = "/api/user/preferences";
pub const MARK_HINT_SHOWN_ROUTE: &str = "/api/user/hint";
pub const GET_MY_METRICS_ROUTE: &str = "/api/user/metrics";

/// Base for the commander maybeboard; the per-card routes hang off it.
pub const COMMANDER_MAYBEBOARD_ROUTE: &str = "/api/user/commander-maybeboard";
pub const GET_COMMANDER_MAYBEBOARD_ROUTE: &str = COMMANDER_MAYBEBOARD_ROUTE;
pub const CLEAR_COMMANDER_MAYBEBOARD_ROUTE: &str = COMMANDER_MAYBEBOARD_ROUTE;

// == card ==

/// Base for card routes that interpolate an id.
pub const CARD_ROUTE: &str = "/api/card";
/// The hour's featured flavor card (unauthed).
pub const FEATURED_FLAVOR_ROUTE: &str = "/api/card/featured-flavor";
pub const SEARCH_CARDS_ROUTE: &str = "/api/card/search";
pub const SEARCH_COMMANDERS_ROUTE: &str = "/api/card/search/commanders";
pub const GET_ARTISTS_ROUTE: &str = "/api/card/artists";
pub const GET_CARD_TYPES_ROUTE: &str = "/api/card/types";
pub const GET_KEYWORDS_ROUTE: &str = "/api/card/keywords";
pub const GET_KEYWORD_REMINDERS_ROUTE: &str = "/api/card/keyword-reminders";
pub const GET_ORACLE_WORDS_ROUTE: &str = "/api/card/oracle-words";
pub const GET_CARD_ROLES_ROUTE: &str = "/api/card/roles";
pub const GET_ORACLE_TAGS_ROUTE: &str = "/api/card/oracle-tags";
/// Universes Beyond franchises offered as exclude-preference exceptions.
pub const GET_UB_FRANCHISES_ROUTE: &str = "/api/card/ub-franchises";
pub const GET_SETS_ROUTE: &str = "/api/card/sets";
pub const GET_LANGUAGES_ROUTE: &str = "/api/card/languages";

// == deck ==

/// Base for deck routes; also the create and list route.
pub const DECK_ROUTE: &str = "/api/deck";
pub const CREATE_DECK_ROUTE: &str = DECK_ROUTE;
pub const GET_DECK_PROFILES_ROUTE: &str = DECK_ROUTE;
pub const GET_DECK_TAGS_ROUTE: &str = "/api/deck/tags";

// == metrics ==

pub const RECORD_USAGE_ROUTE: &str = "/api/metrics/usage";
pub const RECORD_ANONYMOUS_EVENT_ROUTE: &str = "/api/metrics/anonymous";
pub const RECORD_CRASH_ROUTE: &str = "/api/metrics/crash";
pub const PUBLIC_METRICS_ROUTE: &str = "/api/marketing/stats";

// == misc ==

pub const MIN_CLIENT_VERSION_ROUTE: &str = "/api/client/min-version";
pub const CHANGELOG_ROUTE: &str = "/api/changelog";

// == paths carrying an id ==

pub fn get_card_route(scryfall_data_id: Uuid) -> String {
    format!("{CARD_ROUTE}/{scryfall_data_id}")
}

pub fn get_printings_route(oracle_id: Uuid) -> String {
    format!("{CARD_ROUTE}/{oracle_id}/printings")
}

pub fn get_deck_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}")
}

pub fn get_deck_tokens_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/tokens")
}

pub fn get_deck_profile_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/profile/{deck_id}")
}

pub fn update_deck_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}")
}

pub fn delete_deck_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}")
}

pub fn clone_deck_route(source_deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{source_deck_id}/clone")
}

pub fn share_deck_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/share")
}

pub fn get_shared_deck_route(token: Uuid) -> String {
    format!("/api/share/deck/{token}")
}

pub fn clear_deck_suppressions_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/suppressions")
}

pub fn skip_deck_card_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/suppressions")
}

pub fn unskip_deck_card_route(deck_id: Uuid, oracle_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/suppressions/{oracle_id}")
}

pub fn add_commander_maybeboard_card_route(oracle_id: Uuid) -> String {
    format!("{COMMANDER_MAYBEBOARD_ROUTE}/{oracle_id}")
}

pub fn remove_commander_maybeboard_card_route(oracle_id: Uuid) -> String {
    format!("{COMMANDER_MAYBEBOARD_ROUTE}/{oracle_id}")
}

pub fn import_archidekt_deck_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/import/archidekt")
}

pub fn search_deck_cards_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/card/search")
}

pub fn create_deck_card_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/card")
}

pub fn update_deck_card_route(deck_id: Uuid, scryfall_data_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/card/{scryfall_data_id}")
}

pub fn delete_deck_card_route(deck_id: Uuid, scryfall_data_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/card/{scryfall_data_id}")
}

pub fn import_deck_cards_route(deck_id: Uuid) -> String {
    format!("{DECK_ROUTE}/{deck_id}/card/import")
}

/// Every fixed API route: the method that reaches it, and whether it needs a
/// bearer token.
///
/// Derived from the `Endpoint` impls, which is what the clients call, so this
/// is the contract stated as data. `zerver`'s routing test walks it and
/// asserts each entry behaves the way the contract says, because `routes.rs`
/// builds its tree from nested literals and would otherwise agree with these
/// constants only by coincidence.
///
/// The auth flag is what makes that test sharp. A path parameter shadows any
/// unmatched sibling segment (`/api/card/{id}` catches `/api/card/anything`),
/// so "did this 404" cannot tell a missing route from a shadowed one. Knowing
/// a route should answer 401 without a token can.
///
/// Fixed paths only. The id-carrying routes are functions, so the test builds
/// those itself. Health routes are absent: they have no `Endpoint`.
pub const FIXED_ROUTES: &[(Method, &str, bool)] = &[
    (Method::Get, CHANGELOG_ROUTE, false),
    (Method::Patch, CHANGE_EMAIL_ROUTE, true),
    (Method::Patch, CHANGE_PASSWORD_ROUTE, true),
    (Method::Patch, CHANGE_USERNAME_ROUTE, true),
    (Method::Delete, CLEAR_COMMANDER_MAYBEBOARD_ROUTE, true),
    (Method::Post, CREATE_DECK_ROUTE, true),
    (Method::Delete, DELETE_USER_ROUTE, true),
    (Method::Get, FEATURED_FLAVOR_ROUTE, false),
    (Method::Post, FORGOT_PASSWORD_ROUTE, false),
    (Method::Get, GET_ARTISTS_ROUTE, false),
    (Method::Get, GET_CARD_ROLES_ROUTE, false),
    (Method::Get, GET_CARD_TYPES_ROUTE, false),
    (Method::Get, GET_COMMANDER_MAYBEBOARD_ROUTE, true),
    (Method::Get, GET_DECK_PROFILES_ROUTE, true),
    (Method::Get, GET_DECK_TAGS_ROUTE, true),
    (Method::Get, GET_KEYWORDS_ROUTE, false),
    (Method::Get, GET_KEYWORD_REMINDERS_ROUTE, false),
    (Method::Get, GET_LANGUAGES_ROUTE, false),
    (Method::Get, GET_ORACLE_TAGS_ROUTE, false),
    (Method::Get, GET_ORACLE_WORDS_ROUTE, false),
    (Method::Get, GET_SETS_ROUTE, false),
    (Method::Get, GET_UB_FRANCHISES_ROUTE, false),
    (Method::Get, GET_USER_ROUTE, true),
    (Method::Post, LOGIN_ROUTE, false),
    (Method::Post, LOGOUT_ROUTE, true),
    (Method::Patch, MARK_HINT_SHOWN_ROUTE, true),
    (Method::Get, MIN_CLIENT_VERSION_ROUTE, false),
    (Method::Get, PREFERENCES_ROUTE, true),
    (Method::Patch, PREFERENCES_ROUTE, true),
    (Method::Get, PUBLIC_METRICS_ROUTE, false),
    (Method::Post, RECORD_ANONYMOUS_EVENT_ROUTE, false),
    (Method::Post, RECORD_CRASH_ROUTE, false),
    (Method::Post, RECORD_USAGE_ROUTE, true),
    (Method::Post, REFRESH_SESSION_ROUTE, false),
    (Method::Post, REGISTER_ROUTE, false),
    (Method::Post, RESEND_VERIFICATION_ROUTE, true),
    (Method::Post, RESET_PASSWORD_ROUTE, false),
    (Method::Post, SEARCH_CARDS_ROUTE, true),
    (Method::Post, SEARCH_COMMANDERS_ROUTE, true),
    (Method::Post, VERIFY_EMAIL_ROUTE, false),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The id-carrying paths, pinned to their exact shape. These are the
    /// server's registered routes; a shipped client calls whatever is here.
    #[test]
    fn id_paths_render_exactly() {
        let a = Uuid::nil();
        let b = Uuid::from_u128(1);
        let (sa, sb) = (a.to_string(), b.to_string());

        assert_eq!(get_card_route(a), format!("/api/card/{sa}"));
        assert_eq!(get_printings_route(a), format!("/api/card/{sa}/printings"));
        assert_eq!(get_deck_route(a), format!("/api/deck/{sa}"));
        assert_eq!(get_deck_tokens_route(a), format!("/api/deck/{sa}/tokens"));
        assert_eq!(get_deck_profile_route(a), format!("/api/deck/profile/{sa}"));
        assert_eq!(update_deck_route(a), format!("/api/deck/{sa}"));
        assert_eq!(delete_deck_route(a), format!("/api/deck/{sa}"));
        assert_eq!(clone_deck_route(a), format!("/api/deck/{sa}/clone"));
        assert_eq!(share_deck_route(a), format!("/api/deck/{sa}/share"));
        assert_eq!(get_shared_deck_route(a), format!("/api/share/deck/{sa}"));
        assert_eq!(
            clear_deck_suppressions_route(a),
            format!("/api/deck/{sa}/suppressions")
        );
        assert_eq!(
            skip_deck_card_route(a),
            format!("/api/deck/{sa}/suppressions")
        );
        assert_eq!(
            unskip_deck_card_route(a, b),
            format!("/api/deck/{sa}/suppressions/{sb}")
        );
        assert_eq!(
            add_commander_maybeboard_card_route(a),
            format!("/api/user/commander-maybeboard/{sa}")
        );
        assert_eq!(
            remove_commander_maybeboard_card_route(a),
            format!("/api/user/commander-maybeboard/{sa}")
        );
        assert_eq!(
            import_archidekt_deck_route(a),
            format!("/api/deck/{sa}/import/archidekt")
        );
        assert_eq!(
            search_deck_cards_route(a),
            format!("/api/deck/{sa}/card/search")
        );
        assert_eq!(create_deck_card_route(a), format!("/api/deck/{sa}/card"));
        assert_eq!(
            update_deck_card_route(a, b),
            format!("/api/deck/{sa}/card/{sb}")
        );
        assert_eq!(
            delete_deck_card_route(a, b),
            format!("/api/deck/{sa}/card/{sb}")
        );
        assert_eq!(
            import_deck_cards_route(a),
            format!("/api/deck/{sa}/card/import")
        );
    }

    /// Every path is absolute, so `format!("{base}{path}")` and
    /// `Url::set_path` agree.
    #[test]
    fn every_fixed_path_starts_with_a_slash() {
        for (_, path, _) in FIXED_ROUTES {
            assert!(path.starts_with('/'), "{path} is not absolute");
        }
        for path in [HEALTH_ROUTE, SERVER_HEALTH_ROUTE, DATABASE_HEALTH_ROUTE] {
            assert!(path.starts_with('/'), "{path} is not absolute");
        }
    }

    /// The table is hand-extended alongside the consts, so this catches the
    /// cheapest way to get it wrong: naming the same route twice.
    #[test]
    fn the_route_table_has_no_duplicate_pairs() {
        let mut seen: Vec<(Method, &str)> = Vec::new();
        for (method, path, _) in FIXED_ROUTES {
            let pair = (*method, *path);
            assert!(!seen.contains(&pair), "{method:?} {path} is listed twice");
            seen.push(pair);
        }
    }
}
