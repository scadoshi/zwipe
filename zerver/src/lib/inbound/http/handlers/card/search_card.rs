use crate::domain::card::models::{
    scryfall_data::colors::{Color, Colors},
    search_card::{InvalidSearchCards, SearchCards},
};
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{search_card::SearchCardsError, Card},
            ports::CardService,
        },
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
impl From<SearchCardsError> for ApiError {
    fn from(value: SearchCardsError) -> Self {
        value.log_500()
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidSearchCards> for ApiError {
    fn from(value: InvalidSearchCards) -> Self {
        Self::UnprocessableEntity(format!("invalid request: {}", value))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HttpSearchCards {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmc: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmc_range: Option<(f64, f64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_range: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness_range: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_identity_contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl HttpSearchCards {
    pub fn new(
        name: Option<String>,
        type_line: Option<String>,
        set: Option<String>,
        rarity: Option<String>,
        cmc: Option<f64>,
        cmc_range: Option<(f64, f64)>,
        power: Option<i32>,
        power_range: Option<(i32, i32)>,
        toughness: Option<i32>,
        toughness_range: Option<(i32, i32)>,
        color_identity: Option<String>,
        color_identity_contains: Option<String>,
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
            cmc_range,
            power,
            power_range,
            toughness,
            toughness_range,
            color_identity,
            color_identity_contains,
            oracle_text,
            limit,
            offset,
        }
    }

    pub fn by_name(search_query: &str) -> Self {
        HttpSearchCards {
            name: Some(search_query.to_string()),
            type_line: None,
            set: None,
            rarity: None,
            cmc: None,
            cmc_range: None,
            power: None,
            power_range: None,
            toughness: None,
            toughness_range: None,
            color_identity: None,
            color_identity_contains: None,
            oracle_text: None,
            limit: None,
            offset: None,
        }
    }

    pub fn blank() -> Self {
        HttpSearchCards {
            name: None,
            type_line: None,
            set: None,
            rarity: None,
            cmc: None,
            cmc_range: None,
            power: None,
            power_range: None,
            toughness: None,
            toughness_range: None,
            color_identity: None,
            color_identity_contains: None,
            oracle_text: None,
            limit: None,
            offset: None,
        }
    }

    pub fn is_blank(&self) -> bool {
        *self == HttpSearchCards::blank()
    }
}

impl TryFrom<HttpSearchCards> for SearchCards {
    type Error = InvalidSearchCards;
    fn try_from(params: HttpSearchCards) -> Result<Self, Self::Error> {
        let color_identity: Option<Colors> = params.color_identity.map(|s| {
            s.split(',')
                .filter_map(|c| Color::try_from(c).ok())
                .collect()
        });

        let color_identity_contains: Option<Colors> = params.color_identity_contains.map(|s| {
            s.split(',')
                .filter_map(|c| Color::try_from(c).ok())
                .collect()
        });

        SearchCards::new(
            params.name,
            params.type_line,
            params.set,
            params.rarity,
            params.cmc,
            params.cmc_range,
            params.power,
            params.power_range,
            params.toughness,
            params.toughness_range,
            color_identity,
            color_identity_contains,
            params.oracle_text,
            params.limit,
            params.offset,
        )
    }
}

#[cfg(feature = "zerver")]
pub async fn search_cards<AS, US, HS, CS, DS>(
    _: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Query(params): Query<HttpSearchCards>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = SearchCards::try_from(params)?;

    state
        .card_service
        .search_cards(&request)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
