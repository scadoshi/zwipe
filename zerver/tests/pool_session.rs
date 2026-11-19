//! The settings every pooled connection starts with: UTC always, and JIT off
//! when the pool serves requests.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    reason = "a test: helpers outside #[test] fns unwrap, and a panic is the failure report"
)]

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use zwipe::outbound::sqlx::postgres::{Jit, Postgres};

fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap()
}

/// What `statement`, a `SHOW`, reads on one of `pool`'s connections.
async fn show(pool: &PgPool, statement: &'static str) -> String {
    sqlx::query_scalar(statement).fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn a_pool_with_jit_off_starts_each_connection_without_jit_in_utc() {
    let db = Postgres::new(&database_url(), Jit::Off).await.unwrap();
    assert_eq!(show(&db.pool, "SHOW jit").await, "off");
    assert_eq!(show(&db.pool, "SHOW TimeZone").await, "UTC");
}

#[tokio::test]
async fn a_pool_with_jit_on_keeps_the_server_s_jit_setting_in_utc() {
    let plain = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url())
        .await
        .unwrap();
    let server_default = show(&plain, "SHOW jit").await;

    let db = Postgres::new(&database_url(), Jit::On).await.unwrap();
    assert_eq!(show(&db.pool, "SHOW jit").await, server_default);
    assert_eq!(show(&db.pool, "SHOW TimeZone").await, "UTC");
}
