//! Route definitions and path constants shared between frontend and backend.

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

pub use crate::inbound::http::paths::*;

/// Routes that don't require authentication.
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

/// Routes that require `AuthenticatedUser` (JWT Bearer token).
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
        card::{
            get_artists::get_artists, get_card_types::get_card_types,
            get_languages::get_languages, get_sets::get_sets,
        },
        deck::get_deck_profile::get_deck_profile,
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
                    .route("/:scryfall_data_id", get(get_card))
                    .route("/search", post(search_cards))
                    .route("/artists", get(get_artists))
                    .route("/types", get(get_card_types))
                    .route("/languages", get(get_languages))
                    .route("/sets", get(get_sets)),
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
                            .route("/:scryfall_data_id", put(update_deck_card))
                            .route("/:scryfall_data_id", delete(delete_deck_card)),
                    ),
            ),
    )
}
