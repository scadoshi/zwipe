//! Commander maybeboard endpoints (per-user "maybe this commander" list).

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use uuid::Uuid;
use zwipe::inbound::http::routes::{
    add_commander_maybeboard_card_route, clear_commander_maybeboard_route,
    get_commander_maybeboard_route, remove_commander_maybeboard_card_route,
};
use zwipe_core::domain::{auth::models::session::Session, card::Card};

/// Trait for reading and mutating the user's commander maybeboard.
#[allow(missing_docs)]
pub trait ClientCommanderMaybeboard {
    fn get_commander_maybeboard(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;

    fn add_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;

    fn remove_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;

    fn clear_commander_maybeboard(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientCommanderMaybeboard for ZwipeClient {
    async fn get_commander_maybeboard(&self, session: &Session) -> Result<Vec<Card>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_commander_maybeboard_route());
        info!("GET {}", url);

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let cards: Vec<Card> = response.json().await?;
                Ok(cards)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }

    async fn add_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&add_commander_maybeboard_card_route(oracle_id));
        info!("POST {}", url);

        let response = self
            .client
            .post(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }

    async fn remove_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&remove_commander_maybeboard_card_route(oracle_id));
        info!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }

    async fn clear_commander_maybeboard(&self, session: &Session) -> Result<(), ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&clear_commander_maybeboard_route());
        info!("DELETE {}", url);

        let response = self
            .client
            .delete(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
