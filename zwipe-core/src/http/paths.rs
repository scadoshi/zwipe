//! Path constants shared between frontend and backend for URL consistency.
#![allow(missing_docs)]

use uuid::Uuid;

pub fn server_health_route() -> String {
    "/health/server".to_string()
}

pub fn database_health_route() -> String {
    "/health/database".to_string()
}

pub fn register_route() -> String {
    "/api/auth/register".to_string()
}

pub fn login_route() -> String {
    "/api/auth/login".to_string()
}

pub fn refresh_session_route() -> String {
    "/api/auth/refresh".to_string()
}

pub fn get_user_route() -> String {
    "/api/user".to_string()
}

pub fn change_password_route() -> String {
    "/api/user/change-password".to_string()
}

pub fn change_username_route() -> String {
    "/api/user/change-username".to_string()
}

pub fn change_email_route() -> String {
    "/api/user/change-email".to_string()
}

pub fn delete_user_route() -> String {
    "/api/user/delete-user".to_string()
}

pub fn preferences_route() -> String {
    "/api/user/preferences".to_string()
}

pub fn logout_route() -> String {
    "/api/auth/logout".to_string()
}

pub fn verify_email_route() -> String {
    "/api/auth/verify-email".to_string()
}

pub fn forgot_password_route() -> String {
    "/api/auth/forgot-password".to_string()
}

pub fn reset_password_route() -> String {
    "/api/auth/reset-password".to_string()
}

pub fn resend_verification_route() -> String {
    "/api/auth/resend-verification".to_string()
}

pub fn get_card_route(scryfall_data_id: Uuid) -> String {
    format!("/api/card/{}", scryfall_data_id)
}

pub fn get_printings_route(oracle_id: Uuid) -> String {
    format!("/api/card/{}/printings", oracle_id)
}

pub fn search_cards_route() -> String {
    "/api/card/search".to_string()
}

pub fn get_artists_route() -> String {
    "api/card/artists".to_string()
}

pub fn get_card_types_route() -> String {
    "api/card/types".to_string()
}

pub fn get_keywords_route() -> String {
    "api/card/keywords".to_string()
}

pub fn get_oracle_words_route() -> String {
    "api/card/oracle-words".to_string()
}

pub fn get_sets_route() -> String {
    "api/card/sets".to_string()
}

pub fn get_languages_route() -> String {
    "/api/card/languages".to_string()
}

pub fn create_deck_route() -> String {
    "/api/deck".to_string()
}

pub fn get_deck_route(deck_id: Uuid) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn get_deck_tokens_route(deck_id: Uuid) -> String {
    format!("/api/deck/{}/tokens", deck_id)
}

pub fn get_deck_profiles_route() -> String {
    "/api/deck".to_string()
}

pub fn get_deck_profile_route(deck_id: Uuid) -> String {
    format!("/api/deck/profile/{}", deck_id)
}

pub fn update_deck_route(deck_id: Uuid) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn delete_deck_route(deck_id: Uuid) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn clone_deck_route(source_deck_id: Uuid) -> String {
    format!("/api/deck/{}/clone", source_deck_id)
}

pub fn create_deck_card_route(deck_id: Uuid) -> String {
    format!("/api/deck/{}/card", deck_id)
}

pub fn update_deck_card_route(deck_id: Uuid, scryfall_data_id: Uuid) -> String {
    format!("/api/deck/{}/card/{}", deck_id, scryfall_data_id)
}

pub fn delete_deck_card_route(deck_id: Uuid, scryfall_data_id: Uuid) -> String {
    format!("/api/deck/{}/card/{}", deck_id, scryfall_data_id)
}

pub fn import_deck_cards_route(deck_id: Uuid) -> String {
    format!("/api/deck/{}/card/import", deck_id)
}

pub fn record_usage_route() -> String {
    "/api/metrics/usage".to_string()
}

pub fn get_my_metrics_route() -> String {
    "/api/user/metrics".to_string()
}

pub fn public_metrics_route() -> String {
    "/api/marketing/stats".to_string()
}

pub fn min_client_version_route() -> String {
    "/api/client/min-version".to_string()
}
