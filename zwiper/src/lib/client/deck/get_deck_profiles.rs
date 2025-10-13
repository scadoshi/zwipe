use crate::client::auth::{session::ActiveSession, AuthClient};
use dioxus::prelude::*;
use std::future::Future;
use thiserror::Error;
use zwipe::{
    domain::deck::models::deck::{deck_profile::DeckProfile, get_deck_profiles::GetDeckProfiles},
    inbound::http::routes::get_deck_profiles_route,
};

#[derive(Debug, Error)]
pub enum GetDeckProfilesError {
    #[error("thing")]
    Thing,
}

pub trait GetDecks {
    fn get_deck_profiles(
        &self,
        request: GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfilesError>> + Send;
}

impl GetDecks for AuthClient {
    async fn get_deck_profiles(
        &self,
        request: GetDeckProfiles,
    ) -> Result<Vec<DeckProfile>, GetDeckProfilesError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_profiles_route());

        //

        todo!()
    }
}
