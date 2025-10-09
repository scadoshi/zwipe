#[cfg(feature = "zerver")]
use crate::domain::{
    auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
    health::ports::HealthService, user::ports::UserService,
};
#[cfg(feature = "zerver")]
use crate::inbound::http::handlers::auth::{
    change_email, change_password, change_username, delete_user,
};
#[cfg(feature = "zerver")]
use crate::inbound::http::handlers::deck_card::{
    create_deck_card, delete_deck_card, update_deck_card,
};
#[cfg(feature = "zerver")]
use crate::inbound::http::handlers::user::get_user;
#[cfg(feature = "zerver")]
use crate::inbound::http::handlers::{
    auth::{authenticate_user, register_user},
    card::{get_card, search_cards},
    deck::{create_deck_profile, delete_deck, get_deck, update_deck_profile},
    health::{are_server_and_database_running, is_server_running, root},
};
#[cfg(feature = "zerver")]
use crate::inbound::http::AppState;
#[cfg(feature = "zerver")]
use axum::routing::{delete, get, post, put};
#[cfg(feature = "zerver")]
use axum::Router;

// actual routing
#[cfg(feature = "zerver")]
pub fn public_routes<AS, US, HS, CS, DS>() -> Router<AppState<AS, US, HS, CS, DS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    use crate::inbound::http::handlers::auth::refresh_session;

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

// for the frontend to use
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

// actual routing
#[cfg(feature = "zerver")]
pub fn private_routes<AS, US, HS, CS, DS>() -> Router<AppState<AS, US, HS, CS, DS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    Router::new().nest(
        "/api",
        Router::new()
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
                    .route("/search", get(search_cards)),
            )
            .nest(
                "/deck",
                Router::new()
                    .route("/", post(create_deck_profile))
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

// for the frontend to use
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

pub fn get_card_route(card_profile_id: &str) -> String {
    format!("/api/card/{}", card_profile_id)
}

pub fn search_cards_route() -> String {
    "/api/card/search".to_string()
}

pub fn create_deck_route() -> String {
    "/api/deck".to_string()
}

pub fn get_deck_route(deck_id: &str) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn update_deck_route(deck_id: &str) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn delete_deck_route(deck_id: &str) -> String {
    format!("/api/deck/{}", deck_id)
}

pub fn create_deck_card_route(deck_id: &str) -> String {
    format!("/api/deck/{}/card", deck_id)
}

pub fn update_deck_card_route(deck_id: &str, card_profile_id: &str) -> String {
    format!("/api/deck/{}/card/{}", deck_id, card_profile_id)
}

pub fn delete_deck_card_route(deck_id: &str, card_profile_id: &str) -> String {
    format!("/api/deck/{}/card/{}", deck_id, card_profile_id)
}
