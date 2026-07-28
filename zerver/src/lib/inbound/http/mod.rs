//! HTTP layer: Axum server, error mapping, middleware, and route definitions.

/// HTTP request handlers organized by domain.
pub mod handlers;
#[cfg(feature = "zerver")]
/// JWT authentication and last-active tracking middleware.
pub mod middleware;
/// Route definitions mapping paths to handlers.
pub mod routes;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::access_token::JwtSecret,
        auth::ports::{AuthService, ErasedAuthService},
        card::ports::{CardService, ErasedCardService},
        deck::ports::{DeckService, ErasedDeckService},
        health::ports::{ErasedHealthService, HealthService},
        metrics::ports::ErasedMetricsService,
        user::ports::{ErasedUserService, UserService},
    },
    inbound::http::routes::{private_routes, public_routes},
};
#[cfg(feature = "zerver")]
use anyhow::{Context, anyhow};
#[cfg(feature = "zerver")]
use axum::{
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
#[cfg(feature = "zerver")]
use dashmap::DashMap;
#[cfg(feature = "zerver")]
use std::sync::Arc;
#[cfg(feature = "zerver")]
use std::time::Duration;
#[cfg(feature = "zerver")]
use std::time::Instant;
use thiserror::Error;
#[cfg(feature = "zerver")]
use tokio::net;
#[cfg(feature = "zerver")]
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

// =======
//  error
// =======

/// Shared error vocabulary between the server and its clients.
///
/// The server produces these from domain errors and renders them in `IntoResponse`
/// — the **single exit path**: `InternalServerError` logs its carried detail there
/// and sends a generic body, so an unlogged 500 is structurally impossible. Other
/// variants forward their message to the client verbatim (4xx messages are
/// user-safe copy by contract). Clients rebuild the vocabulary from
/// `(status, body)` in their own error layer (zwiper's `ClientError`); transport
/// failures never enter this type.
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
    #[error("{0}")]
    TooManyRequests(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(format!("{value:#}"))
    }
}

impl From<uuid::Error> for ApiError {
    fn from(value: uuid::Error) -> Self {
        Self::UnprocessableEntity(format!("failed to parse uuid: {}", value))
    }
}

#[cfg(feature = "zerver")]
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        // Single exit path: every error response is logged here, tiered by fault.
        // 500s are our fault (error, full detail, generic body); 4xx are client
        // behavior (warn, message is user-safe so it logs and ships verbatim).
        let (status, message) = match self {
            ApiError::InternalServerError(detail) => {
                tracing::error!("{detail}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
            ApiError::UnprocessableEntity(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
            ApiError::TooManyRequests(message) => (StatusCode::TOO_MANY_REQUESTS, message),
        };
        if status != StatusCode::INTERNAL_SERVER_ERROR {
            tracing::warn!("{} {}", status.as_u16(), message);
        }
        (status, message).into_response()
    }
}

/// Converts an error to [`ApiError::InternalServerError`], capturing the full
/// debug repr and a backtrace into the variant while the concrete type still
/// exists. No logging happens here — `IntoResponse` is the single exit path
/// that logs the carried detail before stripping it from the response.
#[cfg(feature = "zerver")]
trait To500 {
    /// Capture diagnostics and convert to [`ApiError::InternalServerError`].
    fn to_500(self) -> ApiError;
}

#[cfg(feature = "zerver")]
impl<E> To500 for E
where
    E: std::error::Error,
{
    fn to_500(self) -> ApiError {
        ApiError::InternalServerError(format!("{:?}\n{}", self, anyhow!("{self}").backtrace()))
    }
}

// ========
//  security headers
// ========

/// Adds security-relevant HTTP response headers to every response.
///
/// - `X-Content-Type-Options: nosniff` — prevents MIME-type sniffing
/// - `X-Frame-Options: DENY` — prevents clickjacking via iframe embedding
/// - `Referrer-Policy: strict-origin-when-cross-origin` — limits referrer leakage
#[cfg(feature = "zerver")]
async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    response
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
    /// Minimum app version allowed (force-update gate); `"0.0.0"` = open.
    pub min_client_version: String,
    /// Public web base URL (e.g. `https://zwipe.net`); used for outbound
    /// User-Agent contact info.
    pub web_base_url: String,
}

/// Shared application state holding all service implementations.
///
/// Services are held as type-erased trait objects (see the `ErasedXService`
/// twins in each domain's ports) so handlers stay free of generic parameters;
/// mock implementations still satisfy the fields via the blanket impls.
#[cfg(feature = "zerver")]
#[derive(Clone)]
#[allow(missing_docs)]
pub struct AppState {
    pub auth_service: Arc<dyn ErasedAuthService>,
    pub user_service: Arc<dyn ErasedUserService>,
    pub health_service: Arc<dyn ErasedHealthService>,
    pub card_service: Arc<dyn ErasedCardService>,
    pub deck_service: Arc<dyn ErasedDeckService>,
    pub metrics_service: Arc<dyn ErasedMetricsService>,
    /// Per-user debounce cache for `users.last_active_at` bumps.
    pub last_active_cache: Arc<DashMap<Uuid, Instant>>,
    /// Minimum app version allowed (force-update gate); `"0.0.0"` = open.
    pub min_client_version: Arc<str>,
    /// Public web base URL (e.g. `https://zwipe.net`); used for outbound
    /// User-Agent contact info.
    pub web_base_url: Arc<str>,
}

/// Axum HTTP server with pre-configured routes and middleware.
#[cfg(feature = "zerver")]
pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

/// Assembles the full Axum router (routes + the middleware stack) from
/// application state. Split out of `HttpServer::new` so integration tests can
/// drive the router directly (`tower::ServiceExt::oneshot`) with no socket bind.
#[cfg(feature = "zerver")]
pub fn build_router(
    state: AppState,
    jwt_secret: JwtSecret,
    allowed_origins: Vec<HeaderValue>,
) -> axum::Router {
    // RequestId is set by SetRequestIdLayer before TraceLayer fires, so it's
    // available as a request extension when we build the span.
    let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
        |request: &axum::extract::Request<_>| {
            let uri = request.uri().to_string();
            let request_id = request
                .extensions()
                .get::<tower_http::request_id::RequestId>()
                .and_then(|id| id.header_value().to_str().ok())
                .unwrap_or("");
            tracing::info_span!(
                "http_request",
                method = ?request.method(),
                uri,
                request_id = %request_id,
            )
        },
    );

    // Layer order is innermost-first, outermost-last. Request flows outer→inner;
    // response flows inner→outer. Effective stack: SetRequestId → PropagateRequestId
    // → trace → CatchPanic → Compression → Cors → security_headers → Timeout(30s)
    // → RequestBodyLimit(2 MiB, innermost).
    let x_request_id = header::HeaderName::from_static("x-request-id");
    axum::Router::new()
        .merge(
            // last-active layer wraps private routes only — it peeks the Bearer
            // token, so it must sit where every request carries one
            private_routes(jwt_secret).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                middleware::track_last_active,
            )),
        )
        .merge(public_routes())
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
        .with_state(state)
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
        metrics_service: Arc<dyn ErasedMetricsService>,
        config: HttpServerConfig<'_>,
    ) -> anyhow::Result<Self> {
        // RequestId is set by SetRequestIdLayer (below) before TraceLayer fires,
        // so it's available as a request extension when we build the span.
        let jwt_secret = auth_service.jwt_secret().clone();
        let state = AppState {
            auth_service: Arc::new(auth_service),
            user_service: Arc::new(user_service),
            health_service: Arc::new(health_service),
            card_service: Arc::new(card_service),
            deck_service: Arc::new(deck_service),
            metrics_service,
            last_active_cache: Arc::new(DashMap::new()),
            min_client_version: Arc::from(config.min_client_version.as_str()),
            web_base_url: Arc::from(config.web_base_url.as_str()),
        };

        let router = build_router(state, jwt_secret, config.allowed_origins);

        let listener = net::TcpListener::bind(&config.bind_address)
            .await
            .with_context(|| format!("failed to listen on {}", config.bind_address))?;

        Ok(Self { router, listener })
    }

    /// Starts serving HTTP requests.
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("listening on {}", self.listener.local_addr()?);
        axum::serve(
            self.listener,
            self.router
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("received error from running server")?;
        Ok(())
    }
}

/// Resolves when the process receives either SIGINT (Ctrl-C, dev) or SIGTERM
/// (systemd `stop`, container orchestrators). `axum::serve` uses this to stop
/// accepting connections and drain in-flight requests before exiting.
#[cfg(feature = "zerver")]
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut stream) = signal(SignalKind::terminate()) {
            stream.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("shutdown signal received (SIGINT)"),
        _ = terminate => tracing::info!("shutdown signal received (SIGTERM)"),
    }
}

#[cfg(all(test, feature = "zerver"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]
    use super::*;
    use http_body_util::BodyExt;

    async fn body_of(error: ApiError) -> (StatusCode, String) {
        let response = error.into_response();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    /// `InternalServerError` carries its diagnostic detail (debug repr,
    /// backtrace) inside the variant so the exit can log it. The exit MUST
    /// strip it: forwarding the message like the 4xx arms do would leak
    /// internals to clients.
    #[tokio::test]
    async fn internal_server_error_body_never_leaks_carried_detail() {
        let detail = "connection refused (os error 111)\nstack backtrace: ...";
        let (status, body) = body_of(ApiError::InternalServerError(detail.to_string())).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, "internal server error");
    }

    #[tokio::test]
    async fn to_500_captures_detail_into_the_variant() {
        let error = std::io::Error::other("db exploded");
        match error.to_500() {
            ApiError::InternalServerError(detail) => assert!(detail.contains("db exploded")),
            other => panic!("expected InternalServerError, got {other:?}"),
        }
    }

    /// 4xx messages are user-safe copy by contract and ship verbatim.
    #[tokio::test]
    async fn four_xx_messages_pass_through_verbatim() {
        let cases = [
            (
                ApiError::UnprocessableEntity("deck name cannot be empty".into()),
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (
                ApiError::Unauthorized("session expired".into()),
                StatusCode::UNAUTHORIZED,
            ),
            (
                ApiError::NotFound("deck not found".into()),
                StatusCode::NOT_FOUND,
            ),
            (
                ApiError::Forbidden("not your deck".into()),
                StatusCode::FORBIDDEN,
            ),
            (
                ApiError::TooManyRequests("slow down".into()),
                StatusCode::TOO_MANY_REQUESTS,
            ),
        ];
        for (error, expected_status) in cases {
            let message = error.to_string();
            let (status, body) = body_of(error).await;
            assert_eq!(status, expected_status);
            assert_eq!(body, message);
        }
    }
}
