//! Integration-test harness.
//!
//! Each test gets a fresh, migrated, isolated database via `#[sqlx::test]`
//! (requires `DATABASE_URL` in the env — `set -a; source zerver/.env`). We build
//! the real Axum router with [`build_router`] over that pool + a capturing fake
//! email sender, and drive it in-process with `tower::ServiceExt::oneshot` — no
//! socket, full middleware stack.

#![allow(clippy::unwrap_used, clippy::indexing_slicing, dead_code)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request, StatusCode, header};
use dashmap::DashMap;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt; // oneshot

use zwipe::domain::auth::models::access_token::JwtSecret;
use zwipe::domain::email::models::{SendEmail, SendEmailError};
use zwipe::domain::email::ports::EmailSender;
use zwipe::domain::{auth, card, deck, health, metrics, user};
use zwipe::inbound::http::{AppState, build_router};
use zwipe::outbound::sqlx::postgres::Postgres;

const TEST_JWT_SECRET: &str = "test-jwt-secret-that-is-at-least-32-characters-long";

/// Captures every outbound email instead of hitting Resend. Later slices read
/// the verify/reset token out of the last captured body.
#[derive(Clone, Default)]
pub struct FakeEmailSender {
    pub sent: Arc<Mutex<Vec<SendEmail>>>,
}

impl EmailSender for FakeEmailSender {
    async fn send_email(&self, email: SendEmail) -> Result<(), SendEmailError> {
        self.sent.lock().unwrap().push(email);
        Ok(())
    }
}

impl FakeEmailSender {
    /// HTML body of the most recently sent email.
    pub fn last_body(&self) -> Option<String> {
        self.sent.lock().unwrap().last().map(|e| e.html_body.clone())
    }
}

// Distinct fake peer IP per TestApp so the per-IP governor limiter never shares
// state across tests running in parallel.
static IP_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct TestApp {
    pub router: axum::Router,
    pub pool: PgPool,
    pub emails: FakeEmailSender,
    fake_ip: SocketAddr,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        let db = Postgres { pool: pool.clone() };
        let emails = FakeEmailSender::default();
        let jwt_secret = JwtSecret::new(TEST_JWT_SECRET).unwrap();

        // Mirrors zerver.rs service construction, with the fake email sender and
        // fixed test config.
        let auth_service = auth::services::Service::new(
            db.clone(),
            db.clone(),
            emails.clone(),
            jwt_secret.clone(),
            "http://localhost".to_string(),
            "support@test.local".to_string(),
        );
        let user_service = user::services::Service::new(db.clone());
        let health_service = health::services::Service::new(db.clone());
        let card_service = card::services::Service::new(db.clone());
        let deck_service = deck::services::Service::new(db.clone(), db.clone());
        let metrics_service: Arc<dyn metrics::ports::ErasedMetricsService> =
            Arc::new(metrics::services::Service::new(db.clone()));

        let state = AppState {
            auth_service: Arc::new(auth_service),
            user_service: Arc::new(user_service),
            health_service: Arc::new(health_service),
            card_service: Arc::new(card_service),
            deck_service: Arc::new(deck_service),
            metrics_service,
            last_active_cache: Arc::new(DashMap::new()),
            min_client_version: Arc::from("0.0.0"),
            web_base_url: Arc::from("http://localhost"),
        };

        let router = build_router(state, jwt_secret, vec![]);

        let n = IP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let o = n.to_be_bytes();
        let fake_ip = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, o[1], o[2], o[3].max(1))),
            40000 + (n as u16 & 0x0fff),
        );

        Self { router, pool, emails, fake_ip }
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        json: Option<Value>,
        token: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(t) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let body = match &json {
            Some(v) => {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                Body::from(serde_json::to_vec(v).unwrap())
            }
            None => Body::empty(),
        };
        let mut request = builder.body(body).unwrap();
        // The public routes' governor key-extractor reads the peer IP from
        // ConnectInfo; oneshot requests have none, so insert a fake one.
        request.extensions_mut().insert(ConnectInfo(self.fake_ip));

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, value)
    }

    pub async fn post(&self, path: &str, json: Value, token: Option<&str>) -> (StatusCode, Value) {
        self.send(Method::POST, path, Some(json), token).await
    }

    pub async fn get(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        self.send(Method::GET, path, None, token).await
    }

    /// Register a fresh user; returns `(access_token, user_id)`.
    pub async fn register(&self, username: &str) -> (String, String) {
        let body = serde_json::json!({
            "username": username,
            "email": format!("{username}@test.local"),
            "password": "TestPass123!",
        });
        let (status, v) = self.post("/api/auth/register", body, None).await;
        assert_eq!(status, StatusCode::CREATED, "register failed: {v}");
        (
            v["access_token"]["value"].as_str().unwrap().to_string(),
            v["user"]["id"].as_str().unwrap().to_string(),
        )
    }
}
