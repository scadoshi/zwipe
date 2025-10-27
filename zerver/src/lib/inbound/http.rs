pub mod handlers;
pub mod helpers;
#[cfg(feature = "zerver")]
pub mod middleware;
pub mod routes;

#[cfg(feature = "zerver")]
use crate::domain::{
    auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
    health::ports::HealthService, user::ports::UserService,
};
#[cfg(feature = "zerver")]
use anyhow::{anyhow, Context};
#[cfg(feature = "zerver")]
use axum::{
    http::{header, HeaderValue, Method, StatusCode},
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

#[derive(Debug, Error)]
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
        use crate::inbound::http::routes::{private_routes, public_routes};

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
        tracing::info!("server running on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("received error from running server")?;
        Ok(())
    }
}
