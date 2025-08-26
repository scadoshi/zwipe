// // In your card handlers
// pub async fn search_cards<CS: CardService>(
//     Query(request): Query<SearchCardRequest>,
//     State(state): State<AppState<AS, US, HS, CS>>,
// ) -> Result<Json<Vec<Card>>, ApiError> {
//     let cards = state
//         .card_service
//         .search_cards(&request)
//         .await
//         .map_err(ApiError::from)?;

//     Ok(Json(cards))
// }
