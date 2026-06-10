//! JWT authentication and last-active tracking middleware.

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::access_token::{JwtSecret, JwtValidate},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::AppState,
};
#[cfg(feature = "zerver")]
use axum::http::header::AUTHORIZATION;
#[cfg(feature = "zerver")]
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
#[cfg(feature = "zerver")]
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
#[cfg(feature = "zerver")]
use std::str::FromStr;
#[cfg(feature = "zerver")]
use std::sync::Arc;
#[cfg(feature = "zerver")]
use std::time::{Duration, Instant};
#[cfg(feature = "zerver")]
use tower_governor::{GovernorError, key_extractor::KeyExtractor};
use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::{
    auth::models::access_token::{Jwt, UserClaims},
    user::username::Username,
};

/// Axum extractor that enforces JWT authentication.
///
/// Including this in a handler signature means the route requires a valid Bearer token.
/// Extraction flow: `Authorization: Bearer <token>` → parse JWT → validate signature
/// → extract claims.
///
/// Rejects with `400 Bad Request` if the header is missing or malformed,
/// `401 Unauthorized` if the signature is invalid.
pub struct AuthenticatedUser {
    /// User ID from JWT claims.
    pub id: Uuid,
    /// Username from JWT claims.
    pub username: Username,
    /// Email from JWT claims.
    pub email: Email,
}

/// Rate-limit key extractor that keys by authenticated user ID from the JWT.
///
/// Used on private routes so each user gets their own rate limit bucket
/// regardless of IP address. Falls back to `UnableToExtractKey` for
/// missing or invalid tokens — the auth middleware rejects those downstream.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct UserIdKeyExtractor {
    jwt_secret: JwtSecret,
}

#[cfg(feature = "zerver")]
impl UserIdKeyExtractor {
    /// Creates a new extractor with the given JWT secret for token validation.
    pub fn new(jwt_secret: JwtSecret) -> Self {
        Self { jwt_secret }
    }
}

#[cfg(feature = "zerver")]
impl KeyExtractor for UserIdKeyExtractor {
    type Key = Uuid;
    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::errors::GovernorError> {
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(GovernorError::UnableToExtractKey)?;

        let jwt = Jwt::from_str(token).map_err(|_| GovernorError::UnableToExtractKey)?;
        let claims = jwt
            .validate(&self.jwt_secret)
            .map_err(|_| GovernorError::UnableToExtractKey)?;

        Ok(claims.user_id)
    }
}

impl From<UserClaims> for AuthenticatedUser {
    fn from(value: UserClaims) -> Self {
        Self {
            id: value.user_id,
            username: value.username,
            email: value.email,
        }
    }
}

/// Debounce window for `users.last_active_at` bumps — at most one DB write
/// per user per window regardless of request volume.
#[cfg(feature = "zerver")]
const LAST_ACTIVE_DEBOUNCE: Duration = Duration::from_secs(60);

/// Bumps `users.last_active_at` for authenticated requests, debounced per user.
///
/// Peeks the Bearer token without enforcing it — missing or invalid tokens
/// pass through untouched and are rejected downstream by the
/// `AuthenticatedUser` extractor. The write is fire-and-forget so it never
/// adds latency to the request path. The debounce cache is in-memory and
/// lost on restart, which is fine: the first request after a restart writes.
#[cfg(feature = "zerver")]
pub async fn track_last_active<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    request: Request,
    next: Next,
) -> Response
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let user_id = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| Jwt::from_str(token).ok())
        .and_then(|jwt| jwt.validate(state.auth_service.jwt_secret()).ok())
        .map(|claims| claims.user_id);

    if let Some(user_id) = user_id {
        let due = state
            .last_active_cache
            .get(&user_id)
            .is_none_or(|last| last.elapsed() >= LAST_ACTIVE_DEBOUNCE);
        if due {
            state.last_active_cache.insert(user_id, Instant::now());
            let metrics = Arc::clone(&state.metrics_service);
            tokio::spawn(async move {
                if let Err(e) = metrics.touch_last_active(user_id).await {
                    tracing::warn!(error = ?e, "metrics: touch_last_active failed");
                }
            });
        }
    }

    next.run(request).await
}

#[cfg(feature = "zerver")]
impl<AS, US, HS, CS, DS> FromRequestParts<AppState<AS, US, HS, CS, DS>> for AuthenticatedUser
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<AS, US, HS, CS, DS>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        let jwt = Jwt::from_str(bearer.token()).map_err(|_| StatusCode::BAD_REQUEST)?;
        let claims = jwt
            .validate(state.auth_service.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser::from(claims))
    }
}
