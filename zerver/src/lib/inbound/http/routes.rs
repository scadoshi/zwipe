#[cfg(feature = "zerver")]
use crate::domain::{
    auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
    health::ports::HealthService, user::ports::UserService,
};
#[cfg(feature = "zerver")]
use crate::inbound::http::handlers::{
    auth::{
        authenticate_user::authenticate_user, change_email::change_email,
        change_password::change_password, change_username::change_username,
        delete_user::delete_user, refresh_session::refresh_session, register_user::register_user,
        revoke_sessions::revoke_sessions,
    },
    card::{get_card::get_card, search_card::search_cards},
    deck::{
        create_deck_profile::create_deck_profile, delete_deck::delete_deck, get_deck::get_deck,
        get_deck_profiles::get_deck_profiles, update_deck_profile::update_deck_profile,
    },
    deck_card::{
        create_deck_card::create_deck_card, delete_deck_card::delete_deck_card,
        update_deck_card::update_deck_card,
    },
    health::{are_server_and_database_running, is_server_running, root},
    user::get_user::get_user,
};
#[cfg(feature = "zerver")]
use crate::inbound::http::AppState;
#[cfg(feature = "zerver")]
use axum::routing::{delete, get, post, put};
#[cfg(feature = "zerver")]
use axum::Router;
use uuid::Uuid;

// ===============
//  public router
// ===============
#[cfg(feature = "zerver")]
pub fn public_routes<AS, US, HS, CS, DS>() -> Router<AppState<AS, US, HS, CS, DS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    Router::new()
        .route("/", get(root))
        .nest(
            "/health",
            Router::new()
                .route("/server", get(is_server_running))
                .route("/database", get(are_server_and_database_running)),
        )
        .nest(
            "/api",
            Router::new().nest(
                "/auth",
                Router::new()
                    .route("/register", post(register_user))
                    .route("/login", post(authenticate_user))
                    .route("/refresh", post(refresh_session)),
            ),
        )
}

// ==============
//  for frontend
// ==============
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

// ================
//  private router
// ================
#[cfg(feature = "zerver")]
pub fn private_routes<AS, US, HS, CS, DS>() -> Router<AppState<AS, US, HS, CS, DS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    use crate::inbound::http::handlers::{
        card::get_card_types::get_card_types, deck::get_deck_profile::get_deck_profile,
    };

    Router::new().nest(
        "/api",
        Router::new()
            .nest(
                "/auth",
                Router::new().route("/logout", post(revoke_sessions)),
            )
            .nest(
                "/user",
                Router::new()
                    .route("/", get(get_user))
                    .route("/change-password", put(change_password))
                    .route("/change-username", put(change_username))
                    .route("/change-email", put(change_email))
                    .route("/delete-user", delete(delete_user)),
            )
            .nest(
                "/card",
                Router::new()
                    .route("/:card_profile_id", get(get_card))
                    .route("/search", get(search_cards))
                    .route("/types", get(get_card_types)),
            )
            .nest(
                "/deck",
                Router::new()
                    .route("/", post(create_deck_profile))
                    .route("/", get(get_deck_profiles))
                    .route("/profile/:deck_id", get(get_deck_profile))
                    .route("/:deck_id", get(get_deck))
                    .route("/:deck_id", put(update_deck_profile))
                    .route("/:deck_id", delete(delete_deck))
                    .nest(
                        "/:deck_id/card",
                        Router::new()
                            .route("/", post(create_deck_card))
                            .route("/:card_profile_id", put(update_deck_card))
                            .route("/:card_profile_id", delete(delete_deck_card)),
                    ),
            ),
    )
}

// ==============
//  for frontend
// ==============
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

pub fn logout_route() -> String {
    "/api/auth/logout".to_string()
}

pub fn get_card_route(card_profile_id: &Uuid) -> String {
    format!("/api/card/{}", card_profile_id)
}

pub fn search_cards_route() -> String {
    "/api/card/search".to_string()
}

pub fn get_all_card_types_route() -> String {
    "api/card/all-types".to_string()
}

pub fn create_deck_route() -> String {
    "/api/deck".to_string()
}

pub fn get_deck_route(deck_id: &Uuid) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn get_deck_profiles_route() -> String {
    format!("/api/deck")
}

pub fn get_deck_profile_route(deck_id: &Uuid) -> String {
    format!("/api/deck/profile/{}", deck_id)
}

pub fn update_deck_route(deck_id: &Uuid) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn delete_deck_route(deck_id: &Uuid) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn create_deck_card_route(deck_id: &Uuid) -> String {
    format!("/api/deck/{}/card", deck_id)
}

pub fn update_deck_card_route(deck_id: &Uuid, card_profile_id: &Uuid) -> String {
    format!("/api/deck/{}/card/{}", deck_id, card_profile_id)
}

pub fn delete_deck_card_route(deck_id: &Uuid, card_profile_id: &Uuid) -> String {
    format!("/api/deck/{}/card/{}", deck_id, card_profile_id)
}
