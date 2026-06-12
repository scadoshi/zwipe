//! Route definitions and path constants shared between frontend and backend.

#[cfg(feature = "zerver")]
use crate::domain::auth::models::access_token::JwtSecret;
#[cfg(feature = "zerver")]
use crate::domain::{
    auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
    health::ports::HealthService, user::ports::UserService,
};
#[cfg(feature = "zerver")]
use crate::inbound::http::AppState;
#[cfg(feature = "zerver")]
use crate::inbound::http::handlers::{
    auth::{
        authenticate_user::authenticate_user, change_email::change_email,
        change_password::change_password, change_username::change_username,
        delete_user::delete_user, refresh_session::refresh_session, register_user::register_user,
        request_password_reset::request_password_reset, resend_verification::resend_verification,
        reset_password::reset_password, revoke_sessions::revoke_sessions,
        verify_email::verify_email,
    },
    card::{
        get_artists::get_artists, get_card::get_card, get_card_types::get_card_types,
        get_keywords::get_keywords, get_languages::get_languages,
        get_oracle_words::get_oracle_words, get_printings::get_printings,
        get_sets::get_sets, search_card::search_cards,
    },
    client::get_min_client_version,
    deck::{
        clone_deck::clone_deck, create_deck_profile::create_deck_profile,
        delete_deck::delete_deck, get_deck::get_deck, get_deck_profile::get_deck_profile,
        get_deck_profiles::get_deck_profiles, get_deck_tokens::get_deck_tokens,
        import_archidekt::import_archidekt_deck, search_deck_cards::search_deck_cards,
        update_deck_profile::update_deck_profile,
    },
    deck_card::{
        create_deck_card::create_deck_card, delete_deck_card::delete_deck_card,
        import_deck_cards::import_deck_cards, update_deck_card::update_deck_card,
    },
    health::{are_server_and_database_running, is_server_running, root},
    metrics::{
        get_my_metrics::get_my_metrics, get_public_metrics::get_public_metrics,
        record_usage::record_usage,
    },
    user::{
        get_preferences::get_preferences, get_user::get_user,
        mark_hint_shown::mark_hint_shown, update_preferences::update_preferences,
    },
};
#[cfg(feature = "zerver")]
use crate::inbound::http::middleware::UserIdKeyExtractor;
#[cfg(feature = "zerver")]
use axum::Router;
#[cfg(feature = "zerver")]
use axum::routing::{delete, get, post, put};
#[cfg(feature = "zerver")]
use std::{sync::Arc, time::Duration};
#[cfg(feature = "zerver")]
use tower_governor::{
    GovernorLayer, errors::GovernorError, governor::GovernorConfigBuilder,
    key_extractor::PeerIpKeyExtractor,
};
#[cfg(feature = "zerver")]
use axum::{
    body::Body,
    http::{Response, StatusCode},
};

/// Rate-limit error handler for the routes keyed by user id
/// (`UserIdKeyExtractor`).
///
/// On those routes the governor layer runs *before* the `AuthenticatedUser`
/// extractor, and it calls the key extractor to bucket the request. When the
/// request has no valid Bearer token the extractor returns
/// `GovernorError::UnableToExtractKey`, and tower_governor's default maps that
/// to a **500** — so an unauthenticated call to a private route answered 500
/// instead of 401, polluting error logs and misleading health probes.
///
/// This remaps that one case to **401 Unauthorized** (the honest status for
/// "you didn't authenticate"). Every other variant — notably
/// `TooManyRequests` (429) for genuine rate-limit hits — is delegated to the
/// library's default response, so real rate limiting is unchanged.
///
/// Only attached to the user-id-keyed limiters; the public, IP-keyed limiters
/// can always extract a key and never hit this path.
#[cfg(feature = "zerver")]
fn unauthorized_on_missing_key(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::UnableToExtractKey => {
            let mut response = Response::new(Body::from("missing or invalid authorization"));
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            response
        }
        other => other.into_response().map(Body::from),
    }
}

pub use zwipe_core::http::paths::*;

/// Routes that don't require authentication.
#[cfg(feature = "zerver")]
#[allow(clippy::expect_used)]
pub fn public_routes<AS, US, HS, CS, DS>() -> Router<AppState<AS, US, HS, CS, DS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    // 5 req / 30s — tight limit, brute-force target
    let login_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(6))
            .burst_size(5)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 5 req / 1hr — rarely needed legitimately
    let register_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(720))
            .burst_size(5)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 20 req / 1min — clients refresh on cold start
    let refresh_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(3))
            .burst_size(20)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 5 req / 1hr per IP — password reset is rare
    let forgot_password_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(720))
            .burst_size(5)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 10 req / 1hr per IP — verify-email + reset-password
    let verify_reset_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(360))
            .burst_size(10)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 60 req / min per IP — card metadata is public and CF-cached, but a tight
    // limit guards the origin from someone bypassing CF to scrape directly.
    let public_card_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(1))
            .burst_size(60)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — public marketing aggregates. CF cache absorbs most
    // traffic; this guards origin if someone bypasses CF or warms many POPs.
    let public_marketing_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — min-version gate poll. ~30-byte payload hit once a
    // minute per app; modest cap guards origin without risking lockout noise.
    let public_client_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );

    Router::new()
        .route("/", get(root))
        .nest(
            "/health",
            Router::new()
                .route("/", get(are_server_and_database_running))
                .route("/server", get(is_server_running))
                .route("/database", get(are_server_and_database_running)),
        )
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/auth",
                    Router::new()
                        .route(
                            "/register",
                            post(register_user).layer(GovernorLayer::new(register_config)),
                        )
                        .route(
                            "/login",
                            post(authenticate_user).layer(GovernorLayer::new(login_config)),
                        )
                        .route(
                            "/refresh",
                            post(refresh_session).layer(GovernorLayer::new(refresh_config)),
                        )
                        .route(
                            "/verify-email",
                            post(verify_email)
                                .layer(GovernorLayer::new(Arc::clone(&verify_reset_config))),
                        )
                        .route(
                            "/forgot-password",
                            post(request_password_reset)
                                .layer(GovernorLayer::new(forgot_password_config)),
                        )
                        .route(
                            "/reset-password",
                            post(reset_password).layer(GovernorLayer::new(verify_reset_config)),
                        ),
                )
                .nest(
                    "/card",
                    Router::new()
                        .route("/{scryfall_data_id}", get(get_card))
                        .route("/{oracle_id}/printings", get(get_printings))
                        .route("/artists", get(get_artists))
                        .route("/types", get(get_card_types))
                        .route("/keywords", get(get_keywords))
                        .route("/oracle-words", get(get_oracle_words))
                        .route("/languages", get(get_languages))
                        .route("/sets", get(get_sets))
                        .layer(GovernorLayer::new(public_card_config)),
                )
                .nest(
                    "/marketing",
                    Router::new()
                        .route("/stats", get(get_public_metrics))
                        .layer(GovernorLayer::new(public_marketing_config)),
                )
                .nest(
                    "/client",
                    Router::new()
                        .route("/min-version", get(get_min_client_version))
                        .layer(GovernorLayer::new(public_client_config)),
                ),
        )
}

/// Routes that require `AuthenticatedUser` (JWT Bearer token).
#[cfg(feature = "zerver")]
#[allow(clippy::expect_used)]
pub fn private_routes<AS, US, HS, CS, DS>(
    jwt_secret: JwtSecret,
) -> Router<AppState<AS, US, HS, CS, DS>>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    // 500 req / 5min (~1.67/s avg) — generous for swiping, keyed by user ID
    let private_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_millis(600))
            .burst_size(500)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // burst 2, then 1 req/30min — account mutations are done once; 2 attempts covers typos
    let sensitive_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(1800))
            .burst_size(2)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // burst 20, then 1 req/10s — commander autocomplete needs headroom for fast typers
    let card_search_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(10))
            .burst_size(20)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // burst 1, then 1 req/60s — resend verification: a fast multi-click sends
    // one email, the rest get 429. Window matches the client cooldown timer.
    let resend_verification_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(60))
            .burst_size(1)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 12 req/min — clients flush usage every ~30s; this gives ample headroom
    let metrics_usage_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(5))
            .burst_size(12)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );

    Router::new()
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/auth",
                    Router::new()
                        .route("/logout", post(revoke_sessions))
                        .route(
                            "/resend-verification",
                            post(resend_verification).layer(
                                GovernorLayer::new(resend_verification_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        ),
                )
                .nest(
                    "/user",
                    Router::new()
                        .route("/", get(get_user))
                        .route(
                            "/change-password",
                            put(change_password)
                                .layer(
                                    GovernorLayer::new(Arc::clone(&sensitive_config))
                                        .error_handler(unauthorized_on_missing_key),
                                ),
                        )
                        .route(
                            "/change-username",
                            put(change_username)
                                .layer(
                                    GovernorLayer::new(Arc::clone(&sensitive_config))
                                        .error_handler(unauthorized_on_missing_key),
                                ),
                        )
                        .route(
                            "/change-email",
                            put(change_email)
                                .layer(
                                    GovernorLayer::new(Arc::clone(&sensitive_config))
                                        .error_handler(unauthorized_on_missing_key),
                                ),
                        )
                        .route(
                            "/delete-user",
                            delete(delete_user).layer(
                                GovernorLayer::new(sensitive_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route("/preferences", get(get_preferences).put(update_preferences))
                        .route("/hint", put(mark_hint_shown))
                        .route("/metrics", get(get_my_metrics)),
                )
                .nest(
                    "/card",
                    Router::new().route(
                        "/search",
                        post(search_cards).layer(
                            GovernorLayer::new(card_search_config)
                                .error_handler(unauthorized_on_missing_key),
                        ),
                    ),
                )
                .nest(
                    "/metrics",
                    Router::new().route(
                        "/usage",
                        post(record_usage).layer(
                            GovernorLayer::new(metrics_usage_config)
                                .error_handler(unauthorized_on_missing_key),
                        ),
                    ),
                )
                .nest(
                    "/deck",
                    Router::new()
                        .route("/", get(get_deck_profiles).post(create_deck_profile))
                        .route("/{deck_id}/import/archidekt", post(import_archidekt_deck))
                        .route("/profile/{deck_id}", get(get_deck_profile))
                        .route(
                            "/{deck_id}",
                            get(get_deck).put(update_deck_profile).delete(delete_deck),
                        )
                        .route("/{deck_id}/clone", post(clone_deck))
                        .route("/{deck_id}/tokens", get(get_deck_tokens))
                        .nest(
                            "/{deck_id}/card",
                            Router::new()
                                .route("/", post(create_deck_card))
                                .route("/import", post(import_deck_cards))
                                .route("/search", post(search_deck_cards))
                                .route(
                                    "/{scryfall_data_id}",
                                    put(update_deck_card).delete(delete_deck_card),
                                ),
                        ),
                ),
        )
        .layer(GovernorLayer::new(private_config).error_handler(unauthorized_on_missing_key))
}
