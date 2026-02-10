//! HTTP layer: Axum server, error mapping, middleware, and route definitions.

/// HTTP request handlers organized by domain.
pub mod handlers;
/// Partial-update helpers (`Optdate`).
pub mod helpers;
#[cfg(feature = "zerver")]
/// JWT authentication middleware.
pub mod middleware;
/// Path constants shared between frontend and backend.
pub mod paths;
/// Route definitions mapping paths to handlers.
pub mod routes;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
        health::ports::HealthService, user::ports::UserService,
    },
    inbound::http::routes::{private_routes, public_routes},
};
#[cfg(feature = "zerver")]
use anyhow::{Context, anyhow};
#[cfg(feature = "zerver")]
use axum::{
    http::{HeaderValue, Method, StatusCode, header},
    response::IntoResponse,
};
#[cfg(feature = "zerver")]
use std::sync::Arc;
use thiserror::Error;
#[cfg(feature = "zerver")]
use tokio::net;
#[cfg(feature = "zerver")]
use tower_http::cors::CorsLayer;

// =======
//  error
// =======

/// Maps domain errors to HTTP status codes.
///
/// `InternalServerError` **strips the original message** in its `IntoResponse` impl,
/// returning a generic `"internal server error"` to prevent leaking internals.
/// Other variants forward the message to the client.
#[derive(Debug, Error, Clone)]
#[allow(missing_docs)]
pub enum ApiError {
    #[error("{0}")]
    InternalServerError(String),
    #[error("{0}")]
    UnprocessableEntity(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("network error: {0}")]
    Network(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Network(format!("json error: {}", value))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(value.to_string())
    }
}

impl From<uuid::Error> for ApiError {
    fn from(value: uuid::Error) -> Self {
        Self::UnprocessableEntity(format!("failed to parse uuid: {}", value))
    }
}

impl From<(reqwest::StatusCode, String)> for ApiError {
    fn from(value: (reqwest::StatusCode, String)) -> Self {
        let (status, message) = value;
        match status {
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Self::InternalServerError(message),
            reqwest::StatusCode::UNAUTHORIZED => Self::Unauthorized(message),
            reqwest::StatusCode::FORBIDDEN => Self::Forbidden(message),
            reqwest::StatusCode::NOT_FOUND => Self::NotFound(message),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY => Self::UnprocessableEntity(message),
            _ => Self::InternalServerError(message),
        }
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
            ApiError::Network(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
            ApiError::UnprocessableEntity(message) => {
                (StatusCode::UNPROCESSABLE_ENTITY, message).into_response()
            }
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, message).into_response(),
        }
    }
}

/// Logs the full error with backtrace, then returns a generic 500 to the client.
///
/// Prevents leaking internal details while preserving diagnostics in server logs.
#[cfg(feature = "zerver")]
trait Log500 {
    /// Log the error and convert to [`ApiError::InternalServerError`].
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

/// Bind address and CORS origins for the HTTP server.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    /// Address to bind the TCP listener to (e.g. `"0.0.0.0:3000"`).
    pub bind_address: &'a str,
    /// Origins permitted by CORS policy.
    pub allowed_origins: Vec<HeaderValue>,
}

/// Shared application state holding all service implementations.
///
/// Generic over five service traits so handlers can be tested with mock implementations.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
#[allow(missing_docs)]
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

/// Axum HTTP server with pre-configured routes and middleware.
#[cfg(feature = "zerver")]
pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

#[cfg(feature = "zerver")]
impl HttpServer {
    /// Builds routes, applies tracing and CORS middleware, and binds the TCP listener.
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

    /// Starts serving HTTP requests.
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!(
            "server running on {}",
            self.listener.local_addr().map_err(|e| anyhow!(e))?
        );
        axum::serve(self.listener, self.router)
            .await
            .context("received error from running server")?;
        Ok(())
    }
}
