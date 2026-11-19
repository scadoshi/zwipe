//! zerver's database connections run with JIT off: its request queries are short and indexed, and
//! compiling their plans cost more than running them. zervice and zerver-admin keep the server's
//! setting, since their bulk aggregates are where JIT can pay off.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(clippy::unwrap_used, reason = "a test: a panic is the failure report")]

use zwipe::outbound::sqlx::postgres::{Postgres, Workload};

/// What `SHOW jit` reads on one of `db`'s connections.
async fn jit(db: &Postgres) -> String {
    sqlx::query_scalar("SHOW jit")
        .fetch_one(&db.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn request_connections_turn_jit_off_and_batch_ones_keep_the_server_s_setting() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let requests = Postgres::new(&url, Workload::Requests).await.unwrap();
    let batch = Postgres::new(&url, Workload::Batch).await.unwrap();
    let plain = sqlx::PgPool::connect(&url).await.unwrap();
    let server: String = sqlx::query_scalar("SHOW jit")
        .fetch_one(&plain)
        .await
        .unwrap();

    assert_eq!(jit(&requests).await, "off", "zerver's connections");
    assert_eq!(
        jit(&batch).await,
        server,
        "zervice's and zerver-admin's connections"
    );
}
