//! Serve the keyword-reminder catalog (name → plain-language reminder).
//!
//! The reminder table lives in `zwipe_core::domain::card::keyword` and is
//! compiled into every binary, but apps ship on store trains, so serving the
//! server's copy lets definition fixes land on deploy (the oracle-tag catalog
//! precedent). The map covers every keyword the database actually serves,
//! resolved through the same function clients fall back to when offline.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use reqwest::StatusCode;
use zwipe_core::domain::card::keyword::keyword_reminder;

use crate::inbound::http::{ApiError, AppState};

/// Returns every distinct keyword name mapped to its reminder text.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_keyword_reminders(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HashMap<String, String>>), ApiError> {
    let keywords = state
        .card_service
        .get_keywords()
        .await
        .map_err(ApiError::from)?;

    let reminders: HashMap<String, String> = keywords
        .into_iter()
        .map(|name| {
            let reminder = keyword_reminder(&name).to_string();
            (name, reminder)
        })
        .collect();

    Ok((StatusCode::OK, Json(reminders)))
}
