pub mod handlers;
pub mod middleware;
pub mod responses;
pub mod scryfall;

use std::sync::Arc;

use anyhow::Context;
use axum::http::{header, HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use tokio::net;
use tower_http::cors::CorsLayer;

use crate::domain::auth::ports::AuthService;
use crate::domain::health::ports::HealthService;
use crate::domain::user::ports::UserService;
use crate::inbound::http::handlers::auth::{authenticate_user, register_user};
use crate::inbound::http::handlers::health::{
    are_server_and_database_running, is_server_running, root,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    pub bind_address: &'a str,
    pub allowed_origins: Vec<HeaderValue>,
}

#[derive(Debug, Clone)]
pub struct AppState<AS: AuthService, US: UserService, HS: HealthService> {
    pub auth_service: Arc<AS>,
    pub user_service: Arc<US>,
    pub health_service: Arc<HS>,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new(
        auth_service: impl AuthService,
        user_service: impl UserService,
        health_service: impl HealthService,
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
            .with_context(|| format!("Failed to listen on {}", config.bind_address))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::debug!("Running on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("Received error from running server")?;
        Ok(())
    }
}

pub fn private_routes<AS: AuthService, US: UserService, HS: HealthService>(
) -> Router<AppState<AS, US, HS>> {
    Router::new()
    // .nest(
    //     "/api/v1",
    //     Router::new().route("/decks", get(handlers::decks::get_decks::<US>)),
    // )
}

pub fn public_routes<AS: AuthService, US: UserService, HS: HealthService>(
) -> Router<AppState<AS, US, HS>> {
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
