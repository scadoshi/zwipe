//! Import a deck from an Archidekt URL.
//!
//! The client sends only the deck URL; the server extracts the id, fetches the
//! deck from Archidekt's public API, resolves each printing by Scryfall id, and
//! creates a new deck owned by the caller. Keeping fetch + parse server-side
//! means the (undocumented) Archidekt shape can be patched without an app release.

#[cfg(feature = "zerver")]
use std::sync::Arc;

#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};

#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::{HttpArchidektImportResult, HttpImportArchidektDeck};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{models::deck::import_archidekt::ImportArchidektError, ports::DeckService},
        health::ports::HealthService,
        metrics::models::kinds::EventKind,
        user::ports::UserService,
    },
    inbound::http::{
        handlers::metrics::check_completion::check_deck_completion,
        middleware::AuthenticatedUser, ApiError, AppState, Log500,
    },
    outbound::archidekt::{ArchidektClient, ArchidektError},
};

#[cfg(feature = "zerver")]
impl From<ArchidektError> for ApiError {
    fn from(value: ArchidektError) -> Self {
        match value {
            ArchidektError::NotFound => Self::NotFound(
                "deck not found on archidekt — make sure it's public and the url is correct"
                    .to_string(),
            ),
            ArchidektError::Upstream(code) => {
                tracing::warn!(status = code, "archidekt upstream error");
                Self::InternalServerError("failed to fetch deck from archidekt".to_string())
            }
            ArchidektError::Network(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<ImportArchidektError> for ApiError {
    fn from(value: ImportArchidektError) -> Self {
        match value {
            ImportArchidektError::InvalidProfile(e) => ApiError::from(e),
            ImportArchidektError::CreateDeck(e) => ApiError::from(e),
            ImportArchidektError::Insert(e) => ApiError::from(e),
            ImportArchidektError::Database(e) => e.log_500(),
        }
    }
}

/// Imports an Archidekt deck into a new deck owned by the authenticated user.
#[cfg(feature = "zerver")]
pub async fn import_archidekt_deck<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpImportArchidektDeck>,
) -> Result<(StatusCode, Json<HttpArchidektImportResult>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let deck_id = ArchidektClient::extract_deck_id(&body.url).ok_or_else(|| {
        ApiError::UnprocessableEntity("could not parse an archidekt deck id from the url".to_string())
    })?;

    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();

    let fetched = ArchidektClient::new().fetch_deck(deck_id).await?;

    let result = state
        .deck_service
        .import_archidekt_deck(user.id, &fetched, email_verified)
        .await
        .map_err(ApiError::from)?;

    // Fire-and-forget metrics: count the new deck and check if it's complete.
    let new_deck_id = result.deck_id;
    let uid = user.id;
    let metrics = Arc::clone(&state.metrics_service);
    tokio::spawn(async move {
        if let Err(e) = metrics.increment_decks_created(uid).await {
            tracing::warn!(error = ?e, "metrics: increment decks_created failed");
        }
        if let Err(e) = metrics
            .record_event(uid, EventKind::DeckCreated, Some(new_deck_id))
            .await
        {
            tracing::warn!(error = ?e, "metrics: record deck_created event failed");
        }
    });
    let deck_service = Arc::clone(&state.deck_service);
    let metrics = Arc::clone(&state.metrics_service);
    tokio::spawn(check_deck_completion(deck_service, metrics, uid, new_deck_id));

    let response = HttpArchidektImportResult {
        deck_id: result.deck_id,
        deck_name: result.deck_name,
        format: result.format,
        command_zone: result.command_zone,
        imported: result.imported,
        unresolved: result.unresolved,
    };
    Ok((StatusCode::CREATED, Json(response)))
}
