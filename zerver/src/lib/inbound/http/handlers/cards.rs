use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{
                scryfall_card::ScryfallCard, GetCardError, GetCardRequest, InvalidUuid,
                SearchCardError, SearchCardRequest,
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

impl From<InvalidUuid> for ApiError {
    fn from(_value: InvalidUuid) -> Self {
        Self::UnprocessableEntity("failed to parse Uuid".to_string())
    }
}

pub async fn get_card<AS, US, HS, CS>(
    State(state): State<AppState<AS, US, HS, CS>>,
    Path(request): Path<GetCardRequest>,
    _: AuthenticatedUser,
) -> Result<ApiSuccess<ScryfallCard>, ApiError>
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
pub struct SearchCardQueryParams {
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

impl TryFrom<SearchCardQueryParams> for SearchCardRequest {
    type Error = SearchCardError;
    fn try_from(params: SearchCardQueryParams) -> Result<Self, Self::Error> {
        let color_identity = params
            .color_identity
            .map(|s| s.split(',').map(|c| c.trim().to_string()).collect());

        Ok(SearchCardRequest::new(
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
    Query(params): Query<SearchCardQueryParams>,
    _: AuthenticatedUser,
) -> Result<ApiSuccess<Vec<ScryfallCard>>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
{
    let request = SearchCardRequest::try_from(params)?;

    state
        .card_service
        .search_scryfall_cards(&request)
        .await
        .map_err(ApiError::from)
        .map(|cards| ApiSuccess::new(StatusCode::OK, cards))
}
