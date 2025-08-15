pub mod card;

pub fn new_pool() -> Result<PgPool, Error> {
    PgPoolOptions::new()
        .min_connections(2)
        .max_connections(10)
        .idle_timeout(Some(Duration::from_secs(300)))
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .map_err(|e| anyhow!("Failed to build connection. Error: {:?}", e))?;
    info!("Generated database connection pool")
}
