use axum::Json;
use reqwest::StatusCode;
use zwipe_core::domain::deck::{DeckTag, DeckTagView};

/// Returns the full deck-tag catalog (slug, display name, description, seed
/// otags), built straight from the `DeckTag` enum, no DB read. Server-delivered
/// so a new deck tag or seed relationship reaches clients on deploy, without an
/// app release (see server_driven_catalogs.md, Part C).
pub async fn get_deck_tags() -> (StatusCode, Json<Vec<DeckTagView>>) {
    let tags: Vec<DeckTagView> = DeckTag::all().iter().map(DeckTag::to_view).collect();
    (StatusCode::OK, Json(tags))
}
