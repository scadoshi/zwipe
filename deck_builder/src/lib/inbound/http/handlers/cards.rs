// // In your card handlers
// pub async fn search_cards<CS: CardService>(
//     Query(params): Query<CardSearchParameters>, // Axum magic! 🪄
//     State(state): State<AppState<AS, US, HS, CS>>,
// ) -> Result<Json<Vec<Card>>, ApiError> {
//     let cards = state
//         .card_service
//         .search_cards(&params)
//         .await
//         .map_err(ApiError::from)?;

//     Ok(Json(cards))
// }
