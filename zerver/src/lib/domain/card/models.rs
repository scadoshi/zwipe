use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    card::models::{card_profile::CardProfile, scryfall_data::ScryfallData},
    deck::models::deck_card::DeckCard,
};

pub mod card_profile;
pub mod scryfall_data;
pub mod sync_metrics;

// =======
//  error
// =======

#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("scryfall data inserted but database returned invalid object: {0}")]
    InvalidScryfallDataFromDatabase(anyhow::Error),
    #[error("card profile created but database returned invalid object: {0}")]
    InvalidCardProfileFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum GetCardError {
    #[error("scryfall data not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("scryfall data found but database returned invalid object: {0}")]
    InvalidScryfallDataFromDatabase(anyhow::Error),
}

#[derive(Debug, Error)]
pub enum InvalidGetCards {
    #[error("invalid id: {0}")]
    InvalidUuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

impl From<uuid::Error> for InvalidGetCards {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidUuid(value)
    }
}

#[derive(Debug, Error)]
pub enum SearchCardError {
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("card profile found but database returned invalid object: {0}")]
    InvalidCardProfileFromDatabase(anyhow::Error),
    #[error("scryfall data found but database returned invalid object: {0}")]
    InvalidScryfallDataFromDatabase(anyhow::Error),
}

// ==========
//  requests
// ==========

#[derive(Debug)]
pub struct GetCard(Uuid);

impl GetCard {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GetCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer).map_err(|e| {
            serde::de::Error::custom(format!(
                "failed to deserialize into string: {}",
                e.to_string()
            ))
        })?;
        GetCard::new(&id)
            .map_err(|e| serde::de::Error::custom(format!("invalid uuid: {}", e.to_string())))
    }
}

pub struct GetCards(Vec<Uuid>);

impl GetCards {
    pub fn new(ids: Vec<&str>) -> Result<Self, InvalidGetCards> {
        if ids.is_empty() {
            return Err(InvalidGetCards::MissingIds);
        }
        Ok(Self(
            ids.into_iter()
                .map(|x| Uuid::try_parse(x))
                .collect::<Result<Vec<Uuid>, uuid::Error>>()?,
        ))
    }

    pub fn ids(&self) -> &Vec<Uuid> {
        &self.0
    }
}

impl From<&[CardProfile]> for GetCards {
    fn from(value: &[CardProfile]) -> Self {
        Self(
            value
                .into_iter()
                .map(|x| x.scryfall_data_id.to_owned())
                .collect(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCard {
    pub name: Option<String>,
    pub type_line: Option<String>,
    pub set: Option<String>,
    pub rarity: Option<String>,
    pub cmc: Option<f64>,
    pub color_identity: Option<Vec<String>>,
    pub oracle_text: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for SearchCard {
    fn default() -> Self {
        Self {
            name: None,
            type_line: None,
            set: None,
            rarity: None,
            cmc: None,
            color_identity: None,
            oracle_text: None,
            limit: Some(20), // default page size
            offset: Some(0), // start at beginning
        }
    }
}

impl SearchCard {
    pub fn new(
        name: Option<String>,
        type_line: Option<String>,
        set: Option<String>,
        rarity: Option<String>,
        cmc: Option<f64>,
        color_identity: Option<Vec<String>>,
        oracle_text: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Self {
        Self {
            name,
            type_line,
            set,
            rarity,
            cmc,
            color_identity,
            oracle_text,
            limit,
            offset,
        }
    }

    pub fn has_filters(&self) -> bool {
        self.name.is_some()
            || self.type_line.is_some()
            || self.set.is_some()
            || self.rarity.is_some()
            || self.cmc.is_some()
            || self.color_identity.is_some()
            || self.oracle_text.is_some()
            || self.limit.is_some()
            || self.offset.is_some()
    }
}

// ======
//  main
// ======

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Card {
    card_profile: CardProfile,
    scryfall_data: ScryfallData,
}

impl Card {
    pub fn new(card_profile: CardProfile, scryfall_data: ScryfallData) -> Self {
        Self {
            card_profile,
            scryfall_data,
        }
    }
}

// =========
//  helpers
// =========

pub trait Sleeve {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card>;
}

impl Sleeve for Vec<ScryfallData> {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card> {
        let mut data_map: HashMap<Uuid, ScryfallData> = self
            .into_iter()
            .map(|sfd| (sfd.id.to_owned(), sfd))
            .collect();

        card_profiles
            .into_iter()
            .filter_map(|cp| {
                data_map
                    .remove(&cp.scryfall_data_id)
                    .map(|sfd| Card::new(cp, sfd))
            })
            .collect::<Vec<Card>>()
    }
}
