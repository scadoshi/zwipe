//! Universes Beyond franchise catalog handler.

use axum::Json;
use reqwest::StatusCode;
use zwipe_core::domain::card::scryfall_data::universe;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;

/// Returns the Universes Beyond franchises the exceptions picker offers, built
/// straight from the compiled franchise table, no DB read.
///
/// Served rather than compiled into the app: the set lists behind each
/// franchise are hand-maintained and grow with every crossover release, so a
/// new franchise becomes selectable on a deploy instead of a store train.
/// Sorted by display name here so every client shows the same order.
pub async fn get_ub_franchises() -> (StatusCode, Json<Vec<UbFranchiseView>>) {
    (StatusCode::OK, Json(universe::selectable_franchise_views()))
}
