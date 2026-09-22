//! Path constants shared between frontend and backend for URL consistency.
//!
//! Fixed paths are `const`; only the ones interpolating an id are functions.
//! Every path is absolute, so both `Url::set_path` and a plain
//! `format!("{base}{path}")` join produce the same URL.
#![allow(missing_docs)]

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
        let fixed = [
            HEALTH_ROUTE,
            SERVER_HEALTH_ROUTE,
            DATABASE_HEALTH_ROUTE,
            REGISTER_ROUTE,
            LOGIN_ROUTE,
            REFRESH_SESSION_ROUTE,
            LOGOUT_ROUTE,
            VERIFY_EMAIL_ROUTE,
            FORGOT_PASSWORD_ROUTE,
            RESET_PASSWORD_ROUTE,
            RESEND_VERIFICATION_ROUTE,
            GET_USER_ROUTE,
            CHANGE_PASSWORD_ROUTE,
            CHANGE_USERNAME_ROUTE,
            CHANGE_EMAIL_ROUTE,
            DELETE_USER_ROUTE,
            PREFERENCES_ROUTE,
            MARK_HINT_SHOWN_ROUTE,
            GET_MY_METRICS_ROUTE,
            COMMANDER_MAYBEBOARD_ROUTE,
            CARD_ROUTE,
            FEATURED_FLAVOR_ROUTE,
            SEARCH_CARDS_ROUTE,
            SEARCH_COMMANDERS_ROUTE,
            GET_ARTISTS_ROUTE,
            GET_CARD_TYPES_ROUTE,
            GET_KEYWORDS_ROUTE,
            GET_KEYWORD_REMINDERS_ROUTE,
            GET_ORACLE_WORDS_ROUTE,
            GET_CARD_ROLES_ROUTE,
            GET_ORACLE_TAGS_ROUTE,
            GET_UB_FRANCHISES_ROUTE,
            GET_SETS_ROUTE,
            GET_LANGUAGES_ROUTE,
            DECK_ROUTE,
            GET_DECK_TAGS_ROUTE,
            RECORD_USAGE_ROUTE,
            RECORD_ANONYMOUS_EVENT_ROUTE,
            RECORD_CRASH_ROUTE,
            PUBLIC_METRICS_ROUTE,
            MIN_CLIENT_VERSION_ROUTE,
            CHANGELOG_ROUTE,
        ];
        for path in fixed {
            assert!(path.starts_with('/'), "{path} is not absolute");
        }
    }
}
