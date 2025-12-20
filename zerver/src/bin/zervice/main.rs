pub mod auth;
pub mod card;

use crate::{auth::CheckSessions, card::CheckCards};
use chrono::NaiveDateTime;
use std::time::Duration;
use zwipe::{
    config::Config,
    domain::{
        auth::services::Service as AuthService, card::services::Service as CardService, logo,
    },
    outbound::sqlx::postgres::Postgres,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logo::Zervice::print();

    let config = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_max_level(config.rust_log)
        .init();

    let db = Postgres::new(&config.database_url).await?;

    let card_service = CardService::new(db.clone());
    let auth_service = AuthService::new(db.clone(), db.clone(), config.jwt_secret);
    let mut latest_token_clean_up: Option<NaiveDateTime> = None;

    tracing::info!("running card migration and refresh token services");
    loop {
        card_service.check_cards().await?;
        auth_service
            .check_sessions(&mut latest_token_clean_up)
            .await?;
        let one_hour = Duration::from_secs(3600);
        tokio::time::sleep(one_hour).await;
    }
}
