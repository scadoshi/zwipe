use axum::http::StatusCode;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use tracing::error;

// define DbPool from the more complex type
type DbPool = Pool<ConnectionManager<PgConnection>>;
type PooledConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn connect_to(pool: DbPool) -> Result<PooledConn, StatusCode> {
    pool.get().map_err(|e| {
        error!(
            "Failed to get connection to database connection pool with error: {:?}",
            e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
