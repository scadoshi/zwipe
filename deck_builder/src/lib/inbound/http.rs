pub mod handlers;
pub mod middleware;
pub mod scryfall;

use std::sync::Arc;

use anyhow::Context;
use axum::http::{header, HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use tokio::net;
use tower_http::cors::CorsLayer;

use crate::domain::user::ports::UserService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    pub bind_address: &'a str,
    pub allowed_origins: Vec<HeaderValue>,
}

#[derive(Debug, Clone)]
struct AppState<US: UserService> {
    user_service: Arc<US>,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new(
        user_service: impl UserService,
        config: HttpServerConfig<'_>,
    ) -> anyhow::Result<Self> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState {
            user_service: Arc::new(user_service),
        };

        let router = axum::Router::new()
            // .merge(private_routes())
            .merge(public_routes())
            .layer(trace_layer)
            .layer(
                CorsLayer::new()
                    .allow_origin(config.allowed_origins)
                    .allow_methods([Method::GET, Method::POST])
                    .allow_headers([header::CONTENT_TYPE]),
            )
            .with_state(state);

        let listener = net::TcpListener::bind(&config.bind_address).await?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::debug!("Listening on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("Received error from running server")?;

        Ok(())
    }
}

// pub fn private_routes<US: UserService>() -> Router<AppState<US>> {
//     Router::new().nest(
//         "/api/v1",
//         Router::new().route("/decks", get(handlers::decks::get_decks::<US>)),
//     )
// }

pub fn public_routes<US: UserService>() -> Router<AppState<US>> {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health_check))
        .route("/health/deep", get(handlers::health::health_check_deep))
        .nest(
            "/api/v1",
            Router::new()
                .route("/auth/login", post(handlers::auth::login::<US>))
                .route("/auth/register", post(handlers::auth::register::<US>)),
        )
}
