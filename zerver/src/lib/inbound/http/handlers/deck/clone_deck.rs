#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::{
                clone_deck::CloneDeckError,
                get_deck_profile::GetDeckProfileError,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        metrics::models::kinds::EventKind,
        user::ports::UserService,
    },
    inbound::http::{
        handlers::metrics::check_completion::check_deck_completion,
        middleware::AuthenticatedUser, ApiError, AppState, Log500,
    },
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::clone_deck::{CloneDeck, InvalidCloneDeck};
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::{HttpCloneDeck, HttpClonedDeck};

#[cfg(feature = "zerver")]
impl From<CloneDeckError> for ApiError {
    fn from(value: CloneDeckError) -> Self {
        match value {
            CloneDeckError::SourceNotFound => Self::NotFound("source deck not found".to_string()),
            CloneDeckError::Forbidden => {
                Self::Forbidden("cannot clone another user's deck".to_string())
            }
            CloneDeckError::Duplicate => Self::UnprocessableEntity(
                "a deck with that name already exists".to_string(),
            ),
            CloneDeckError::LimitReached => Self::UnprocessableEntity(
                "deck limit reached, verify your email to unlock more".to_string(),
            ),
            // GetDeckProfileError has no blanket From<_> for ApiError in scope,
            // so map its variants explicitly here. NotFound / Forbidden are
            // already peeled off into dedicated CloneDeckError variants by
            // the service, but we handle them defensively.
            CloneDeckError::GetSource(e) => match e {
                GetDeckProfileError::NotFound => {
                    Self::NotFound("source deck not found".to_string())
                }
                GetDeckProfileError::Forbidden => {
                    Self::Forbidden("cannot clone another user's deck".to_string())
                }
                GetDeckProfileError::Database(e) => e.log_500(),
                GetDeckProfileError::DeckProfileFromDb(e) => e.log_500(),
            },
            CloneDeckError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidCloneDeck> for ApiError {
    fn from(value: InvalidCloneDeck) -> Self {
        match value {
            InvalidCloneDeck::Name(e) => {
                Self::UnprocessableEntity(format!("invalid deck name: {}", e))
            }
        }
    }
}

/// Clones an existing deck owned by the authenticated user.
///
/// The source deck id is taken from the URL path; the new deck name comes
/// from the JSON body. The response contains only the new deck's id —
/// the client navigates to the deck view which loads the full aggregate.
#[cfg(feature = "zerver")]
pub async fn clone_deck<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(source_deck_id): Path<Uuid>,
    Json(body): Json<HttpCloneDeck>,
) -> Result<(StatusCode, Json<HttpClonedDeck>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    // Fetch email_verified the same way create_deck_profile does (handler line 71).
    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();

    let request = CloneDeck::new(source_deck_id, body.new_name, user.id, email_verified)?;

    let new_deck_id = state
        .deck_service
        .clone_deck(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    tokio::spawn(async move {
        if let Err(e) = metrics.increment_decks_created(uid).await {
            tracing::warn!(error = ?e, "metrics: increment decks_created failed (clone)");
        }
        if let Err(e) = metrics
            .record_event(uid, EventKind::DeckCreated, Some(new_deck_id))
            .await
        {
            tracing::warn!(error = ?e, "metrics: record deck_created event failed (clone)");
        }
        // A clone is a complete copy of an existing deck, so it may already be
        // valid — run the completion check immediately.
        check_deck_completion(deck_service, metrics, uid, new_deck_id).await;
    });

    Ok((
        StatusCode::CREATED,
        Json(HttpClonedDeck { deck_id: new_deck_id }),
    ))
}
