//! HTTP layer: Axum server, error mapping, middleware, and route definitions.

/// HTTP request handlers organized by domain.
pub mod handlers;
#[cfg(feature = "zerver")]
/// JWT authentication middleware.
pub mod middleware;
/// Route definitions mapping paths to handlers.
pub mod routes;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
        health::ports::HealthService, metrics::ports::ErasedMetricsService,
        user::ports::UserService,
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
use std::sync::Arc;
use thiserror::Error;
#[cfg(feature = "zerver")]
use tokio::net;
#[cfg(feature = "zerver")]
use std::time::Duration;
#[cfg(feature = "zerver")]
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};

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
    #[error("{0}")]
    TooManyRequests(String),
}

impl ApiError {
    /// Returns a safe, user-facing message — never leaks internal details like URLs or stack traces.
    pub fn to_user_message(&self) -> String {
        match self {
            ApiError::Network(_) => {
                "connection error — check your network and try again".to_string()
            }
            ApiError::InternalServerError(_) => {
                "something went wrong — please try again".to_string()
            }
            other => other.to_string(),
        }
    }
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
        let message = message.to_lowercase();
        match status {
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => Self::InternalServerError(message),
            reqwest::StatusCode::UNAUTHORIZED => Self::Unauthorized(message),
            reqwest::StatusCode::FORBIDDEN => Self::Forbidden(message),
            reqwest::StatusCode::NOT_FOUND => Self::NotFound(message),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY => Self::UnprocessableEntity(message),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Self::TooManyRequests(message),
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
            ApiError::Network(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response(),
            ApiError::UnprocessableEntity(message) => {
                (StatusCode::UNPROCESSABLE_ENTITY, message).into_response()
            }
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, message).into_response(),
            ApiError::TooManyRequests(message) => {
                (StatusCode::TOO_MANY_REQUESTS, message).into_response()
            }
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
}

/// Shared application state holding all service implementations.
///
/// Generic over five service traits so handlers can be tested with mock implementations.
#[cfg(feature = "zerver")]
#[derive(Clone)]
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
    pub metrics_service: Arc<dyn ErasedMetricsService>,
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
        metrics_service: Arc<dyn ErasedMetricsService>,
        config: HttpServerConfig<'_>,
    ) -> anyhow::Result<Self> {
        // RequestId is set by SetRequestIdLayer (below) before TraceLayer fires,
        // so it's available as a request extension when we build the span.
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

        let jwt_secret = auth_service.jwt_secret().clone();
        let state = AppState {
            auth_service: Arc::new(auth_service),
            user_service: Arc::new(user_service),
            health_service: Arc::new(health_service),
            card_service: Arc::new(card_service),
            deck_service: Arc::new(deck_service),
            metrics_service,
        };

        // Layer order is innermost-first, outermost-last. Request flows
        // outer→inner; response flows inner→outer. Effective stack:
        //
        //   SetRequestIdLayer       (outermost — generates UUID, sets header
        //                            and extension on the request)
        //   PropagateRequestIdLayer (captures the id from the request so it
        //                            can copy it onto the response on the way
        //                            out; must be inside Set, before trace)
        //   trace_layer             (reads request_id from extensions into the
        //                            span so every log line carries it)
        //   CatchPanicLayer         (turn handler panics into 500s)
        //   CompressionLayer        (gzip/br response bodies based on Accept-Encoding;
        //                            outside CORS so headers ride along uncompressed)
        //   CorsLayer
        //   security_headers
        //   TimeoutLayer            (30s per request; rejects with 408)
        //   RequestBodyLimitLayer   (innermost — 2 MiB cap on request bodies)
        let x_request_id = header::HeaderName::from_static("x-request-id");
        let router = axum::Router::new()
            .merge(private_routes(jwt_secret))
            .merge(public_routes())
            .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
            .layer(TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_secs(30),
            ))
            .layer(axum::middleware::from_fn(security_headers))
            .layer(
                CorsLayer::new()
                    .allow_origin(config.allowed_origins)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
            )
            .layer(CompressionLayer::new())
            .layer(CatchPanicLayer::new())
            .layer(trace_layer)
            .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
            .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
            .with_state(state);

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
