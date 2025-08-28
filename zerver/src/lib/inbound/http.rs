// internal
pub mod handlers;
pub mod middleware;
pub mod responses;
pub mod scryfall;
use crate::domain::auth::ports::AuthService;
use crate::domain::card::ports::CardService;
use crate::domain::health::ports::HealthService;
use crate::domain::user::ports::UserService;
use crate::inbound::http::handlers::auth::{authenticate_user, register_user};
use crate::inbound::http::handlers::cards::{get_card, search_cards};
use crate::inbound::http::handlers::health::{
    are_server_and_database_running, is_server_running, root,
};
// std
use std::sync::Arc;
// external
use anyhow::Context;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net;
use tower_http::cors::CorsLayer;

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
pub struct AppState<AS, US, HS, CS>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    pub auth_service: Arc<AS>,
    pub user_service: Arc<US>,
    pub health_service: Arc<HS>,
    pub card_service: Arc<CS>,
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
        };

        let router = axum::Router::new()
            .merge(private_routes())
            .merge(public_routes())
            .layer(trace_layer)
            .layer(
                CorsLayer::new()
                    .allow_origin(config.allowed_origins)
                    .allow_methods([Method::GET, Method::POST])
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

// ===========
//  api error
// ===========

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

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponseBody::new_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )),
            )
                .into_response(),

            ApiError::UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponseBody::new_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    message,
                )),
            )
                .into_response(),

            ApiError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponseBody::new_error(
                    StatusCode::UNAUTHORIZED,
                    message,
                )),
            )
                .into_response(),

            ApiError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ApiResponseBody::new_error(StatusCode::NOT_FOUND, message)),
            )
                .into_response(),
        }
    }
}

// =============
//  api success
// =============

#[derive(Debug)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T: Serialize + PartialEq> PartialEq for ApiSuccess<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 .0 == other.1 .0
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
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
pub struct ApiErrorData {
    pub message: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    fn new(status_code: StatusCode, data: T) -> Self {
        ApiResponseBody {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

// ========
//  routes
// ========

pub fn private_routes<AS, US, HS, CS>() -> Router<AppState<AS, US, HS, CS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/cards/:id", get(get_card))
            .route("/cards/search", get(search_cards)),
    )
}

pub fn public_routes<AS, US, HS, CS>() -> Router<AppState<AS, US, HS, CS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    Router::new()
        .route("/", get(root))
        .route("/health/server", get(is_server_running))
        .route("/health/database", get(are_server_and_database_running))
        .nest(
            "/api/v1",
            Router::new()
                .route("/auth/register", post(register_user))
                .route("/auth/login", post(authenticate_user)),
        )
}
