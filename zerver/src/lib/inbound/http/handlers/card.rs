use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{
                card_profile::GetCardProfileError, scryfall_data::GetScryfallDataError, Card,
                GetCard, GetCardError, InvalidSearchCard, SearchCard, SearchCardError,
            },
            ports::CardService,
        },
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

// =====
//  get
// =====

impl From<GetCardError> for ApiError {
    fn from(value: GetCardError) -> Self {
        match value {
            GetCardError::GetCardProfileError(GetCardProfileError::NotFound) => {
                Self::NotFound("card profile not found".to_string())
            }

            GetCardError::GetScryfallDataError(GetScryfallDataError::NotFound) => {
                Self::NotFound("scryfall data not found".to_string())
            }

            e => e.log_500(),
        }
    }
}

pub async fn get_card<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(request): Path<GetCard>,
    _: AuthenticatedUser,
) -> Result<(StatusCode, Json<Card>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .card_service
        .get_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|card| (StatusCode::OK, Json(card)))
}

// ========
//  search
// ========

impl From<SearchCardError> for ApiError {
    fn from(value: SearchCardError) -> Self {
        value.log_500()
    }
}

impl From<InvalidSearchCard> for ApiError {
    fn from(value: InvalidSearchCard) -> Self {
        Self::UnprocessableEntity(format!("invalid request: {}", value))
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
    type Error = InvalidSearchCard;
    fn try_from(params: SearchCardRawParameters) -> Result<Self, Self::Error> {
        let color_identity = params
            .color_identity
            .map(|s| s.split(',').map(|c| c.trim().to_string()).collect());

        SearchCard::new(
            params.name,
            params.type_line,
            params.set,
            params.rarity,
            params.cmc,
            color_identity,
            params.oracle_text,
            params.limit,
            params.offset,
        )
    }
}

pub async fn search_cards<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Query(params): Query<SearchCardRawParameters>,
    _: AuthenticatedUser,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = SearchCard::try_from(params)?;

    state
        .card_service
        .search_cards(&request)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
