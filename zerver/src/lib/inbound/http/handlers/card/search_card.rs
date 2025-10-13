use crate::domain::card::models::{
    scryfall_data::colors::{Color, Colors},
    search_card::{InvalidSearchCard, SearchCard},
};
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{search_card::SearchCardError, Card},
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
use serde::Deserialize;

#[cfg(feature = "zerver")]
impl From<SearchCardError> for ApiError {
    fn from(value: SearchCardError) -> Self {
        value.log_500()
    }
}

#[cfg(feature = "zerver")]
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
}

impl SearchCardRawParameters {
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
}

impl TryFrom<SearchCardRawParameters> for SearchCard {
    type Error = InvalidSearchCard;
    fn try_from(params: SearchCardRawParameters) -> Result<Self, Self::Error> {
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

        SearchCard::new(
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
    Query(params): Query<SearchCardRawParameters>,
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
