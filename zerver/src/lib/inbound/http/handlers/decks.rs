// use axum::{extract::State, http::StatusCode, response::Json};
// use serde_json::{json, Value};
// use sqlx::query_as;
// use tracing::error;

// use crate::{
//     domain::user::ports::UserService,
//     inbound::http::{middleware::AuthenticatedUser, AppState},
// };

// pub async fn get_decks<US: UserService>(
//     State(app_state): State<AppState<US>>,
//     authenticated_user: AuthenticatedUser,
// ) -> Result<Json<Value>, StatusCode> {
//     let user_decks = query_as!(
//         Deck,
//         "SELECT * FROM decks WHERE user_id = $1",
//         authenticated_user.user_id
//     )
//     .fetch_all(&app_state.db_pool)
//     .await
//     .map_err(|e| {
//         error!("Failed to load decks with error: {:?}", e);
//         StatusCode::INTERNAL_SERVER_ERROR
//     })?;

//     Ok(Json(json!({
//         "decks": user_decks,
//         "user_id": authenticated_user.user_id,
//         "count": user_decks.len()
//     })))
// }
