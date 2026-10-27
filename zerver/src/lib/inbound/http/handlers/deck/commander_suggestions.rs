//! Suggested commanders handlers: the ranked list and its dismissals.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use zwipe_core::http::contracts::suggestions::{
    DEFAULT_SUGGESTION_LIMIT, HttpBuilderProfile, HttpClearedDismissals, HttpColorAffinity,
    HttpColorIdentityCount, HttpCommanderSuggestion, HttpCommanderSuggestions,
    HttpDismissCommander, HttpPlayStyleSummary, HttpProfileRole, HttpProfileTag, HttpSharedTag,
    HttpStaple, HttpSuggestionReasons, MAX_SUGGESTION_LIMIT,
};

use crate::domain::deck::models::deck::commander_suggestions::{
    BuilderProfile, ColorAffinity, ColorIdentityCount, CommanderDismissal, CommanderSuggestion,
    CommanderSuggestions, CommanderSuggestionsError, InvalidCommanderDismissal, PlayStyleSummary,
};
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<CommanderSuggestionsError> for ApiError {
    fn from(value: CommanderSuggestionsError) -> Self {
        match value {
            CommanderSuggestionsError::Database(e) => e.to_500(),
        }
    }
}

impl From<InvalidCommanderDismissal> for ApiError {
    fn from(value: InvalidCommanderDismissal) -> Self {
        match value {
            InvalidCommanderDismissal::OracleId(e) => {
                Self::UnprocessableEntity(format!("invalid oracle id: {}", e))
            }
        }
    }
}

impl From<CommanderSuggestions> for HttpCommanderSuggestions {
    fn from(value: CommanderSuggestions) -> Self {
        Self {
            suggestions: value
                .suggestions
                .into_iter()
                .map(HttpCommanderSuggestion::from)
                .collect(),
            deck_count: value.deck_count,
            profile: value.profile.map(HttpBuilderProfile::from),
        }
    }
}

impl From<ColorAffinity> for HttpColorAffinity {
    fn from(value: ColorAffinity) -> Self {
        Self {
            color: value.color,
            decks: value.decks,
        }
    }
}

impl From<ColorIdentityCount> for HttpColorIdentityCount {
    fn from(value: ColorIdentityCount) -> Self {
        Self {
            colors: value.colors,
            decks: value.decks,
        }
    }
}

impl From<PlayStyleSummary> for HttpPlayStyleSummary {
    fn from(value: PlayStyleSummary) -> Self {
        Self {
            deck_count: value.deck_count,
            identities: value
                .identities
                .into_iter()
                .map(HttpColorIdentityCount::from)
                .collect(),
        }
    }
}

impl From<BuilderProfile> for HttpBuilderProfile {
    fn from(value: BuilderProfile) -> Self {
        Self {
            deck_count: value.deck_count,
            colors: value
                .colors
                .into_iter()
                .map(HttpColorAffinity::from)
                .collect(),
            top_identity: value.top_identity.map(HttpColorIdentityCount::from),
            tags: value
                .tags
                .into_iter()
                .map(|tag| HttpProfileTag {
                    slug: tag.slug,
                    role: tag.role,
                    decks: tag.decks,
                    cards: tag.cards,
                    score: tag.score,
                })
                .collect(),
            roles: value
                .roles
                .into_iter()
                .map(|role| HttpProfileRole {
                    role: role.role,
                    decks: role.decks,
                })
                .collect(),
            staples: value
                .staples
                .into_iter()
                .map(|staple| HttpStaple {
                    card: staple.card,
                    decks: staple.decks,
                })
                .collect(),
        }
    }
}

impl From<CommanderSuggestion> for HttpCommanderSuggestion {
    fn from(value: CommanderSuggestion) -> Self {
        Self {
            card: value.card,
            reasons: HttpSuggestionReasons {
                tags: value
                    .tags
                    .into_iter()
                    .map(|tag| HttpSharedTag {
                        slug: tag.slug,
                        role: tag.role,
                    })
                    .collect(),
                colors: value
                    .colors
                    .into_iter()
                    .map(HttpColorAffinity::from)
                    .collect(),
            },
        }
    }
}

/// Query string of the suggestions list.
#[derive(Debug, Deserialize)]
pub struct SuggestionsQuery {
    /// Suggestions wanted; `DEFAULT_SUGGESTION_LIMIT` when absent, clamped to
    /// `MAX_SUGGESTION_LIMIT`.
    pub limit: Option<u32>,
}

/// Returns the authenticated user's ranked commander suggestions.
pub async fn get_commander_suggestions(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(query): Query<SuggestionsQuery>,
) -> Result<(StatusCode, Json<HttpCommanderSuggestions>), ApiError> {
    let limit = query
        .limit
        .unwrap_or(DEFAULT_SUGGESTION_LIMIT)
        .min(MAX_SUGGESTION_LIMIT);
    state
        .deck_service
        .commander_suggestions(user.id, limit)
        .await
        .map_err(ApiError::from)
        .map(|suggestions| (StatusCode::OK, Json(suggestions.into())))
}

/// Returns the authenticated user's deck count and most built color
/// identities, for Home's play style panel.
pub async fn get_play_style_summary(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HttpPlayStyleSummary>), ApiError> {
    state
        .deck_service
        .play_style_summary(user.id)
        .await
        .map_err(ApiError::from)
        .map(|summary| (StatusCode::OK, Json(summary.into())))
}

/// Hides a commander from the authenticated user's suggestions for a week.
pub async fn dismiss_commander(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(body): Json<HttpDismissCommander>,
) -> Result<StatusCode, ApiError> {
    let request = CommanderDismissal::new(user.id, body.oracle_id);
    state
        .deck_service
        .dismiss_commander(&request)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Lifts one of the authenticated user's dismissals (idempotent).
pub async fn undismiss_commander(
    State(state): State<AppState>,
    Path(oracle_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let request = CommanderDismissal::from_path(user.id, &oracle_id)?;
    state
        .deck_service
        .undismiss_commander(&request)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Lifts every dismissal of the authenticated user, answering the count.
pub async fn clear_commander_dismissals(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<(StatusCode, Json<HttpClearedDismissals>), ApiError> {
    let cleared = state
        .deck_service
        .clear_commander_dismissals(user.id)
        .await
        .map_err(ApiError::from)?;
    Ok((
        StatusCode::OK,
        Json(HttpClearedDismissals {
            cleared: u32::try_from(cleared).unwrap_or(u32::MAX),
        }),
    ))
}
