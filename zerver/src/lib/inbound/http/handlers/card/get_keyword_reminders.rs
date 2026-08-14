//! Serve the keyword-reminder catalog (name → plain-language reminder).
//!
//! The reminder table lives in `zwipe_core::domain::card::keyword` and is
//! compiled into every binary — but apps ship on store trains, so serving the
//! server's copy lets definition fixes land on deploy (the oracle-tag catalog
//! precedent). The map covers every keyword the database actually serves,
//! resolved through the same function clients fall back to when offline.

#[cfg(feature = "zerver")]
use crate::inbound::http::{ApiError, AppState};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;
#[cfg(feature = "zerver")]
use std::collections::HashMap;
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::keyword::keyword_reminder;

/// Returns every distinct keyword name mapped to its reminder text.
#[cfg(feature = "zerver")]
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
