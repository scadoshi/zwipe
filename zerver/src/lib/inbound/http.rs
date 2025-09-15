pub mod handlers;
#[cfg(feature = "zerver")]
pub mod middleware;

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
use anyhow::{anyhow, Context};
#[cfg(feature = "zerver")]
use axum::http::{header, HeaderValue, Method, StatusCode};
#[cfg(feature = "zerver")]
use axum::response::IntoResponse;
#[cfg(feature = "zerver")]
use axum::routing::{delete, get, post, put};
#[cfg(feature = "zerver")]
use axum::Router;
#[cfg(feature = "zerver")]
use std::sync::Arc;
#[cfg(feature = "zerver")]
use tokio::net;
#[cfg(feature = "zerver")]
use tower_http::cors::CorsLayer;

// =======
//  error
// =======

#[cfg(feature = "zerver")]
#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
    Unauthorized(String),
    NotFound(String),
}

#[cfg(feature = "zerver")]
impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(value.to_string())
    }
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for ApiError {
    fn from(value: uuid::Error) -> Self {
        Self::UnprocessableEntity(format!("failed to parse `Uuid`: {}", value))
    }
}

#[cfg(feature = "zerver")]
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response(),

            ApiError::UnprocessableEntity(message) => {
                (StatusCode::UNPROCESSABLE_ENTITY, message).into_response()
            }

            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message).into_response(),

            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
        }
    }
}

#[cfg(feature = "zerver")]
trait Log500 {
    fn log_500(self) -> ApiError;
}

#[cfg(feature = "zerver")]
impl<E> Log500 for E
where
    E: std::error::Error,
{
    fn log_500(self) -> ApiError {
        tracing::error!("{:?}\n{}", self, anyhow!("{self}").backtrace());
        ApiError::InternalServerError("internal server error".to_string())
    }
}

// ========
//  server
// ========

#[cfg(feature = "zerver")]
/// contains configuration for the creation of an HttpServer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    pub bind_address: &'a str,
    pub allowed_origins: Vec<HeaderValue>,
}

#[cfg(feature = "zerver")]
/// contains services
#[derive(Debug, Clone)]
pub struct AppState<AS, US, HS, CS, DS>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    pub auth_service: Arc<AS>,
    pub user_service: Arc<US>,
    pub health_service: Arc<HS>,
    pub card_service: Arc<CS>,
    pub deck_service: Arc<DS>,
}

#[cfg(feature = "zerver")]
/// server with a router and a listener
/// for running our application server
pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

#[cfg(feature = "zerver")]
impl HttpServer {
    pub async fn new(
        auth_service: impl AuthService,
        user_service: impl UserService,
        health_service: impl HealthService,
        card_service: impl CardService,
        deck_service: impl DeckService,
        config: HttpServerConfig<'_>,
    ) -> anyhow::Result<Self> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState {
            auth_service: Arc::new(auth_service),
            user_service: Arc::new(user_service),
            health_service: Arc::new(health_service),
            card_service: Arc::new(card_service),
            deck_service: Arc::new(deck_service),
        };

        let router = axum::Router::new()
            .merge(private_routes())
            .merge(public_routes())
            .layer(trace_layer)
            .layer(
                CorsLayer::new()
                    .allow_origin(config.allowed_origins)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([header::CONTENT_TYPE]),
            )
            .with_state(state);

        let listener = net::TcpListener::bind(&config.bind_address)
            .await
            .with_context(|| format!("failed to listen on {}", config.bind_address))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::debug!("server running on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("received error from running server")?;
        Ok(())
    }
}

// ========
//  routes
// ========

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
        .route("/health/server", get(is_server_running))
        .route("/health/database", get(are_server_and_database_running))
        .nest(
            "/api",
            Router::new().nest(
                "/auth",
                Router::new()
                    .route("/register", post(register_user))
                    .route("/login", post(authenticate_user)),
            ),
        )
}
