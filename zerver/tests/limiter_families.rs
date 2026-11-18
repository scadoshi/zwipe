//! Address-keyed rate limits by family: an IPv6 client counts by its /64 and gets double the
//! IPv4 allowance, so a household or office on one IPv6 network has room; IPv4 keeps its own,
//! and an IPv4 address written as IPv6 counts as IPv4.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

mod common;

use std::net::{IpAddr, SocketAddr};

use axum::http::{Method, StatusCode};
use common::TestApp;
use serde_json::json;
use zwipe_core::http::paths::{FORGOT_PASSWORD_ROUTE, HEALTH_ROUTE};

/// A peer at `ip`.
fn from(ip: &str) -> SocketAddr {
    SocketAddr::new(ip.parse::<IpAddr>().unwrap(), 50001)
}

/// How many health checks pass, one from each of `addresses` in turn, before the first 429.
async fn health_burst(app: &TestApp, addresses: impl Fn(u32) -> String) -> u32 {
    for n in 0..200 {
        let (status, _) = app
            .send_from(from(&addresses(n)), Method::GET, HEALTH_ROUTE, None, None)
            .await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            return n;
        }
        assert_eq!(status, StatusCode::OK, "request {n}");
    }
    200
}

#[sqlx::test]
async fn an_ipv4_client_keeps_its_burst(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    assert_eq!(health_burst(&app, |_| "203.0.113.7".to_owned()).await, 30);
}

#[sqlx::test]
async fn an_ipv6_64_gets_double_however_its_addresses_move(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let burst = health_burst(&app, |n| format!("2001:db8:1:2::{:x}", n.saturating_add(1))).await;
    assert_eq!(burst, 60, "twice IPv4's 30, one bucket for the whole /64");
}

#[sqlx::test]
async fn two_64s_dont_share_a_bucket(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    assert_eq!(
        health_burst(&app, |_| "2001:db8:1:2::1".to_owned()).await,
        60
    );
    let (status, _) = app
        .send_from(
            from("2001:db8:1:3::1"),
            Method::GET,
            HEALTH_ROUTE,
            None,
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the next /64 is another client");
}

#[sqlx::test]
async fn an_ipv4_address_written_as_ipv6_counts_as_ipv4(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    assert_eq!(
        health_burst(&app, |_| "::ffff:203.0.113.9".to_owned()).await,
        30
    );
}

/// A limiter that guards a rare, sensitive call doubles only its burst for a /64.
#[sqlx::test]
async fn forgot_password_doubles_its_burst_for_a_64(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let body = json!({ "email": "nobody@example.com" });
    let burst = |ip: &'static str| {
        let app = &app;
        let body = body.clone();
        async move {
            for n in 0..50 {
                let (status, _) = app
                    .send_from(
                        from(ip),
                        Method::POST,
                        FORGOT_PASSWORD_ROUTE,
                        Some(body.clone()),
                        None,
                    )
                    .await;
                if status == StatusCode::TOO_MANY_REQUESTS {
                    return n;
                }
            }
            50
        }
    };
    assert_eq!(burst("203.0.113.7").await, 5);
    assert_eq!(burst("2001:db8:1:2::1").await, 10);
}
