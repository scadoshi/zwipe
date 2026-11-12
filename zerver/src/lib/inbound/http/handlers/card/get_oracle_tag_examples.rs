//! The public oracle tag examples handler: a page of a tag's most played cards for a reader with
//! no account.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag_examples::{OracleTagSlug, examples_query};

use crate::domain::card::ports::CardService;
use crate::inbound::http::{ApiError, AppState};

/// The page's place in the examples: how many come before it.
#[derive(Debug, Deserialize)]
pub struct ExamplesOffset {
    /// How many examples come before this page; absent is the first page.
    #[serde(default)]
    offset: u32,
}

/// Serves one page of `slug`'s example cards from `offset`: public card data only, at most
/// [`EXAMPLES_PAGE_SIZE`](zwipe_core::domain::card::oracle_tag_examples::EXAMPLES_PAGE_SIZE) of
/// them.
///
/// # Errors
///
/// Returns 422 for a malformed slug or an offset past the examples' end, and 500 when the search
/// fails.
pub async fn get_oracle_tag_examples(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(ExamplesOffset { offset }): Query<ExamplesOffset>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    let slug: OracleTagSlug = slug
        .parse()
        .map_err(|e| ApiError::UnprocessableEntity(format!("invalid oracle tag: {e}")))?;
    let query =
        examples_query(&slug, offset).map_err(|e| ApiError::UnprocessableEntity(e.to_string()))?;
    state
        .card_service
        .search_oracle_tag_examples(&query)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
