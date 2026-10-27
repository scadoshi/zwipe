//! Suggested commanders handlers: the ranked list and its dismissals.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use zwipe_core::http::contracts::suggestions::{
    DEFAULT_SUGGESTION_LIMIT, HttpActivity, HttpActivityDay, HttpBuilderProfile,
    HttpClearedDismissals, HttpColorAffinity, HttpColorCards, HttpColorIdentityCount, HttpColorMap,
    HttpColorPair, HttpCommanderSuggestion, HttpCommanderSuggestions, HttpDismissCommander,
    HttpHardPass, HttpPlayStyleSummary, HttpProfileRole, HttpProfileTag, HttpRoleRates,
    HttpSharedTag, HttpStaple, HttpSuggestionReasons, HttpSwipeOutcomes, HttpTagRates,
    MAX_SUGGESTION_LIMIT,
};

use crate::domain::deck::models::deck::activity::{
    Activity, ActivityDay, HardPass, RoleRates, SwipeOutcomes, TagRates,
};
use crate::domain::deck::models::deck::commander_suggestions::{
    BuilderProfile, ColorAffinity, ColorIdentityCount, ColorMap, CommanderDismissal,
    CommanderSuggestion, CommanderSuggestions, CommanderSuggestionsError,
    InvalidCommanderDismissal, PlayStyleSummary,
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

impl From<Activity> for HttpActivity {
    fn from(value: Activity) -> Self {
        Self {
            through: value.through,
            days: value.days.into_iter().map(HttpActivityDay::from).collect(),
            outcomes: value.outcomes.into(),
            roles: value.roles.into_iter().map(HttpRoleRates::from).collect(),
            tags: value.tags.into_iter().map(HttpTagRates::from).collect(),
            passes: value.passes.into_iter().map(HttpHardPass::from).collect(),
        }
    }
}

impl From<ActivityDay> for HttpActivityDay {
    fn from(value: ActivityDay) -> Self {
        Self {
            day: value.day,
            swipes: value.swipes,
            searches: value.searches,
        }
    }
}

impl From<SwipeOutcomes> for HttpSwipeOutcomes {
    fn from(value: SwipeOutcomes) -> Self {
        Self {
            added: value.added,
            skipped: value.skipped,
            maybed: value.maybed,
            removed: value.removed,
        }
    }
}

impl From<RoleRates> for HttpRoleRates {
    fn from(value: RoleRates) -> Self {
        Self {
            role: value.role,
            outcomes: value.outcomes.into(),
        }
    }
}

impl From<TagRates> for HttpTagRates {
    fn from(value: TagRates) -> Self {
        Self {
            tag: value.tag,
            role: value.role,
            outcomes: value.outcomes.into(),
            everyone: value.everyone.into(),
        }
    }
}

impl From<HardPass> for HttpHardPass {
    fn from(value: HardPass) -> Self {
        Self {
            card: value.card,
            skipped: value.skipped,
            commanders: value.commanders,
        }
    }
}

impl From<PlayStyleSummary> for HttpPlayStyleSummary {
    fn from(value: PlayStyleSummary) -> Self {
        Self {
            deck_count: value.deck_count,
            color_map: Some(HttpColorMap::from(value.color_map)),
        }
    }
}

impl From<ColorMap> for HttpColorMap {
    fn from(value: ColorMap) -> Self {
        Self {
            colors: value
                .colors
                .into_iter()
                .map(|c| HttpColorCards {
                    color: c.color,
                    cards: c.cards,
                })
                .collect(),
            colorless_cards: value.colorless_cards,
            pairs: value
                .pairs
                .into_iter()
                .map(|p| HttpColorPair {
                    colors: p.colors,
                    decks: p.decks,
                })
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
            color_map: Some(HttpColorMap::from(value.color_map)),
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

/// Returns the authenticated user's deck count and color map, for Home's
/// play style panel.
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

/// Returns the authenticated user's activity for the Activity screen.
pub async fn get_activity(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HttpActivity>), ApiError> {
    state
        .deck_service
        .activity(user.id)
        .await
        .map_err(ApiError::from)
        .map(|activity| (StatusCode::OK, Json(activity.into())))
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
