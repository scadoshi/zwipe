use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{
                scryfall_data::GetScryfallDataError, Card, GetCard, GetCardError, SearchCard,
                SearchCardError,
            },
            ports::CardService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, ApiSuccess, AppState},
};
use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

// =====
//  get
// =====

impl From<GetCardError> for ApiError {
    fn from(value: GetCardError) -> Self {
        match value {
            GetCardError::NotFound => Self::NotFound("card not found".to_string()),
            e => {
                tracing::error!("{:?}\n{}", e, anyhow!("{e}").backtrace());
                Self::InternalServerError("internal server error".to_string())
            }
        }
    }
}

pub async fn get_card<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Path(request): Path<GetCard>,
    _: AuthenticatedUser,
) -> Result<ApiSuccess<Card>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    state
        .card_service
        .get_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|card| ApiSuccess::new(StatusCode::OK, card))
}

// ========
//  search
// ========

impl From<SearchCardError> for ApiError {
    fn from(value: SearchCardError) -> Self {
        tracing::error!("{:?}\n{}", value, anyhow!("{value}").backtrace());
        Self::InternalServerError("internal server error".to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchCardRawParameters {
    name: Option<String>,
    type_line: Option<String>,
    set: Option<String>,
    rarity: Option<String>,
    cmc: Option<f64>,
    color_identity: Option<String>,
    oracle_text: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl TryFrom<SearchCardRawParameters> for SearchCard {
    type Error = SearchCardError;
    fn try_from(params: SearchCardRawParameters) -> Result<Self, Self::Error> {
        let color_identity = params
            .color_identity
            .map(|s| s.split(',').map(|c| c.trim().to_string()).collect());

        Ok(SearchCard::new(
            params.name,
            params.type_line,
            params.set,
            params.rarity,
            params.cmc,
            color_identity,
            params.oracle_text,
            params.limit,
            params.offset,
        ))
    }
}

pub async fn search_cards<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Query(params): Query<SearchCardRawParameters>,
    _: AuthenticatedUser,
) -> Result<ApiSuccess<Vec<Card>>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let request = SearchCard::try_from(params)?;

    state
        .card_service
        .search_cards(&request)
        .await
        .map_err(ApiError::from)
        .map(|cards| ApiSuccess::new(StatusCode::OK, cards))
}
