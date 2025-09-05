pub mod handlers;
pub mod middleware;
pub mod scryfall;
use crate::domain::{
    auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
    health::ports::HealthService, user::ports::UserService,
};
use crate::inbound::http::handlers::{
    auth::{authenticate_user, register_user},
    cards::{get_card, search_cards},
    decks::{create_deck_profile, delete_deck, get_deck, update_deck_profile},
    health::{are_server_and_database_running, is_server_running, root},
};
use anyhow::{anyhow, Context};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;
use tokio::net;
use tower_http::cors::CorsLayer;

// =======
//  error
// =======

#[derive(Debug)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
    Unauthorized(String),
    NotFound(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(value.to_string())
    }
}

impl From<uuid::Error> for ApiError {
    fn from(value: uuid::Error) -> Self {
        Self::UnprocessableEntity(format!("failed to parse `Uuid`: {}", value))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HttpResponse::new_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )),
            )
                .into_response(),

            ApiError::UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(HttpResponse::new_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    message,
                )),
            )
                .into_response(),

            ApiError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(HttpResponse::new_error(StatusCode::UNAUTHORIZED, message)),
            )
                .into_response(),

            ApiError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(HttpResponse::new_error(StatusCode::NOT_FOUND, message)),
            )
                .into_response(),
        }
    }
}

trait Log500 {
    fn log_500(self) -> ApiError;
}

impl<E> Log500 for E
where
    E: std::error::Error,
{
    fn log_500(self) -> ApiError {
        tracing::error!("{:?}\n{}", self, anyhow!("{self}").backtrace());
        ApiError::InternalServerError("internal server error".to_string())
    }
}

// =========
//  success
// =========

#[derive(Debug)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<HttpResponse<T>>);

impl<T: Serialize + PartialEq> PartialEq for ApiSuccess<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 .0 == other.1 .0
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(HttpResponse::new(status, data)))
    }
}

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

// =============
//  http things
// =============

#[derive(Debug, Serialize, PartialEq)]
pub struct HttpError {
    pub message: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct HttpResponse<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> HttpResponse<T> {
    fn new(status_code: StatusCode, data: T) -> Self {
        HttpResponse {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl HttpResponse<HttpError> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: HttpError { message },
        }
    }
}

// ========
//  server
// ========

/// contains configuration for the creation of an HttpServer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    pub bind_address: &'a str,
    pub allowed_origins: Vec<HeaderValue>,
}

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

/// server with a router and a listener
/// for running our application server
pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

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
                "/cards",
                Router::new()
                    .route("/:id", get(get_card))
                    .route("/search", get(search_cards)),
            )
            .nest(
                "/decks",
                Router::new()
                    .route("", post(create_deck_profile))
                    .route("/:id", get(get_deck))
                    .route("/:id", put(update_deck_profile))
                    .route("/:id", delete(delete_deck)),
            ),
    )
}

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
            Router::new()
                .route("/auth/register", post(register_user))
                .route("/auth/login", post(authenticate_user)),
        )
}
