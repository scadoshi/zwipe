//! JWT authentication and last-active tracking middleware.

#[cfg(feature = "zerver")]
use crate::{
    domain::auth::models::access_token::{JwtSecret, JwtValidate},
    inbound::http::AppState,
};
#[cfg(feature = "zerver")]
use axum::http::header::AUTHORIZATION;
#[cfg(feature = "zerver")]
use axum::{
    extract::{ConnectInfo, FromRequestParts, Request, State},
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
use std::net::{IpAddr, SocketAddr};
#[cfg(feature = "zerver")]
use std::str::FromStr;
#[cfg(feature = "zerver")]
use std::sync::Arc;
#[cfg(feature = "zerver")]
use std::time::{Duration, Instant};
#[cfg(feature = "zerver")]
use tower_governor::{GovernorError, key_extractor::KeyExtractor};
use uuid::Uuid;
use zwipe_core::domain::{
    Email,
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

/// Canonical Cloudflare header carrying the true client IP.
#[cfg(feature = "zerver")]
const CF_CONNECTING_IP: &str = "cf-connecting-ip";

/// Rate-limit key extractor that keys by the real client IP behind Cloudflare.
///
/// The server runs behind a Cloudflare Tunnel: `cloudflared` proxies every
/// request from `127.0.0.1`, so the TCP peer address is identical for all
/// external clients. `PeerIpKeyExtractor` keys on that peer, which would place
/// the entire internet in a single shared rate-limit bucket — one client could
/// exhaust it and lock everyone out, and per-attacker brute-force throttling
/// wouldn't work at all.
///
/// Cloudflare sets `CF-Connecting-IP` to the true client IP and overwrites any
/// client-supplied value at its edge, so requests arriving through the tunnel
/// can't forge it. The origin is unreachable directly from the public internet
/// (ufw default-deny inbound; only loopback and `tailscale0` are allowed), so
/// the header is trustworthy here.
///
/// Falls back to the socket peer IP when the header is absent — i.e. for
/// non-Cloudflare paths (localhost health checks, Tailscale admin access),
/// which are trusted. Real internet traffic always carries the header.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct CfConnectingIpKeyExtractor;

#[cfg(feature = "zerver")]
impl KeyExtractor for CfConnectingIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &axum::http::Request<T>) -> Result<Self::Key, GovernorError> {
        if let Some(ip) = req
            .headers()
            .get(CF_CONNECTING_IP)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<IpAddr>().ok())
        {
            return Ok(ip);
        }

        // No Cloudflare header: fall back to the socket peer IP (localhost /
        // Tailscale paths). External traffic always carries the header.
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|info| info.0.ip())
            .ok_or(GovernorError::UnableToExtractKey)
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
pub async fn track_last_active(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
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
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
        let jwt = Jwt::from_str(bearer.token()).map_err(|_| StatusCode::UNAUTHORIZED)?;
        let claims = jwt
            .validate(state.auth_service.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser::from(claims))
    }
}

#[cfg(all(test, feature = "zerver"))]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{CF_CONNECTING_IP, CfConnectingIpKeyExtractor};
    use axum::extract::ConnectInfo;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tower_governor::key_extractor::KeyExtractor;

    fn peer(ip: [u8; 4]) -> ConnectInfo<SocketAddr> {
        ConnectInfo(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
            40000,
        ))
    }

    fn request(
        header: Option<&str>,
        peer_ip: Option<ConnectInfo<SocketAddr>>,
    ) -> axum::http::Request<()> {
        let mut builder = axum::http::Request::builder();
        if let Some(h) = header {
            builder = builder.header(CF_CONNECTING_IP, h);
        }
        let mut req = builder.body(()).unwrap();
        if let Some(p) = peer_ip {
            req.extensions_mut().insert(p);
        }
        req
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn uses_cf_connecting_ip_when_present() {
        // Even though the socket peer is loopback (the tunnel), the real
        // client IP from the header must win.
        let req = request(Some("203.0.113.7"), Some(peer([127, 0, 0, 1])));
        let key = CfConnectingIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, ip("203.0.113.7"));
    }

    #[test]
    fn distinct_cf_ips_yield_distinct_keys() {
        // The core property: two clients behind the same tunnel peer get
        // separate buckets.
        let a = CfConnectingIpKeyExtractor
            .extract(&request(Some("203.0.113.7"), Some(peer([127, 0, 0, 1]))))
            .unwrap();
        let b = CfConnectingIpKeyExtractor
            .extract(&request(Some("198.51.100.4"), Some(peer([127, 0, 0, 1]))))
            .unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn falls_back_to_peer_when_header_absent() {
        // Non-Cloudflare path (e.g. Tailscale): no header, key off the peer.
        let req = request(None, Some(peer([100, 64, 0, 9])));
        let key = CfConnectingIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, ip("100.64.0.9"));
    }

    #[test]
    fn falls_back_to_peer_when_header_garbage() {
        let req = request(Some("not-an-ip"), Some(peer([100, 64, 0, 9])));
        let key = CfConnectingIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, ip("100.64.0.9"));
    }

    #[test]
    fn trims_whitespace_in_header() {
        let req = request(Some("  203.0.113.7  "), Some(peer([127, 0, 0, 1])));
        let key = CfConnectingIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, ip("203.0.113.7"));
    }

    #[test]
    fn errors_when_no_header_and_no_peer() {
        let req = request(None, None);
        assert!(CfConnectingIpKeyExtractor.extract(&req).is_err());
    }
}
