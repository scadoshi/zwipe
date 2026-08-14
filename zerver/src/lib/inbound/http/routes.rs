//! Route definitions and path constants shared between frontend and backend.

#[cfg(feature = "zerver")]
use crate::domain::auth::models::access_token::JwtSecret;
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
        featured_flavor::get_featured_flavor, get_artists::get_artists, get_card::get_card,
        get_card_roles::get_card_roles, get_card_types::get_card_types,
        get_keyword_reminders::get_keyword_reminders, get_keywords::get_keywords,
        get_languages::get_languages, get_oracle_tags::get_oracle_tags,
        get_oracle_words::get_oracle_words, get_printings::get_printings, get_sets::get_sets,
        search_card::search_cards, search_commanders::search_commanders,
    },
    changelog::get_changelog,
    client::get_min_client_version,
    deck::{
        clear_deck_suppressions::clear_deck_suppressions,
        clone_deck::clone_deck,
        create_deck_profile::create_deck_profile,
        delete_deck::delete_deck,
        get_deck::get_deck,
        get_deck_profile::get_deck_profile,
        get_deck_profiles::get_deck_profiles,
        get_deck_tags::get_deck_tags,
        get_deck_tokens::get_deck_tokens,
        get_shared_deck::get_shared_deck,
        import_archidekt::import_archidekt_deck,
        search_deck_cards::search_deck_cards,
        share_deck::{share_deck, unshare_deck},
        skip_deck_card::{skip_deck_card, unskip_deck_card},
        update_deck_profile::update_deck_profile,
    },
    deck_card::{
        create_deck_card::create_deck_card, delete_deck_card::delete_deck_card,
        import_deck_cards::import_deck_cards, patch_deck_card::patch_deck_card,
    },
    health::{are_server_and_database_running, is_server_running, root},
    metrics::{
        get_my_metrics::get_my_metrics, get_public_metrics::get_public_metrics,
        record_anonymous_event::record_anonymous_event, record_crash::record_crash,
        record_usage::record_usage,
    },
    user::{
        commander_maybeboard::{
            add_commander_maybeboard_card, clear_commander_maybeboard, get_commander_maybeboard,
            remove_commander_maybeboard_card,
        },
        get_preferences::get_preferences,
        get_user::get_user,
        mark_hint_shown::mark_hint_shown,
        update_preferences::update_preferences,
    },
};
#[cfg(feature = "zerver")]
use crate::inbound::http::middleware::{CfConnectingIpKeyExtractor, UserIdKeyExtractor};
#[cfg(feature = "zerver")]
use axum::Router;
#[cfg(feature = "zerver")]
use axum::extract::DefaultBodyLimit;
#[cfg(feature = "zerver")]
use axum::routing::{delete, get, patch, post};
#[cfg(feature = "zerver")]
use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Response, StatusCode, header},
};
#[cfg(feature = "zerver")]
use std::{sync::Arc, time::Duration};
#[cfg(feature = "zerver")]
use tower_governor::{GovernorLayer, errors::GovernorError, governor::GovernorConfigBuilder};
#[cfg(feature = "zerver")]
use tower_http::set_header::SetResponseHeaderLayer;

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
/// "you didn't authenticate"). `TooManyRequests` gets the stable 429 body
/// (see [`too_many_requests_response`]); anything else delegates to the
/// library default.
///
/// Only attached to the user-id-keyed limiters; the public, IP-keyed limiters
/// can always extract a key and use [`stable_rate_limit`] instead.
#[cfg(feature = "zerver")]
fn unauthorized_on_missing_key(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::UnableToExtractKey => {
            let mut response = Response::new(Body::from("missing or invalid authorization"));
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            response
        }
        GovernorError::TooManyRequests { wait_time, headers } => {
            too_many_requests_response(wait_time, headers)
        }
        other => other.into_response().map(Body::from),
    }
}

/// Rate-limit error handler for the public, IP-keyed limiters: the stable 429
/// body for `TooManyRequests`, library default for everything else (their key
/// extraction never legitimately fails, so no 401 remap here).
#[cfg(feature = "zerver")]
fn stable_rate_limit(error: GovernorError) -> Response<Body> {
    match error {
        GovernorError::TooManyRequests { wait_time, headers } => {
            too_many_requests_response(wait_time, headers)
        }
        other => other.into_response().map(Body::from),
    }
}

/// 429 with a STABLE body and the exact wait in the `Retry-After` header.
///
/// tower_governor's default body interpolates the live countdown ("Too Many
/// Requests! Wait for 1784s"); the client shows that text in a toast AND uses
/// it in the error-report dedupe key, so every second of a lockout minted a
/// fresh `client_errors` row (field-confirmed 2026-08-05). Bucketed copy keeps
/// the message stable across an incident; machines that want precision read
/// the header.
#[cfg(feature = "zerver")]
fn too_many_requests_response(wait_time: u64, headers: Option<HeaderMap>) -> Response<Body> {
    let mut response = Response::new(Body::from(rate_limit_copy(wait_time)));
    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
    if let Some(limit_headers) = headers {
        response.headers_mut().extend(limit_headers);
    }
    if let Ok(value) = HeaderValue::from_str(&wait_time.to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
}

/// Human copy for a 429, bucketed so it stays identical for minutes at a time
/// (long lockouts round up to 5-minute steps).
#[cfg(feature = "zerver")]
fn rate_limit_copy(wait_secs: u64) -> String {
    match wait_secs {
        0..=60 => "too many requests, try again in a minute".to_string(),
        61..=300 => "too many requests, try again in a few minutes".to_string(),
        _ => format!(
            "too many requests, try again in about {} minutes",
            wait_secs.div_ceil(300) * 5
        ),
    }
}

pub use zwipe_core::http::paths::*;

/// `Cache-Control: public, max-age=3600` for successful responses on the
/// public read endpoints (card catalogs, marketing stats, changelog), so the
/// CF edge (whose cache rules respect origin TTL) holds one copy per hour
/// instead of falling back to its daily default — which held the flavor card
/// ~20h on 2026-08-06. One simple number everywhere; only featured-flavor
/// needs more (its content flips ON the hour, so its handler counts down to
/// the boundary, and `if_not_present` lets that header win).
///
/// Success-only on purpose: without the guard a 429 or error body would leave
/// browser-cacheable for an hour.
#[cfg(feature = "zerver")]
fn hourly_public_cache(response: &Response<Body>) -> Option<HeaderValue> {
    response
        .status()
        .is_success()
        .then(|| HeaderValue::from_static("public, max-age=3600"))
}

#[cfg(feature = "zerver")]
type CacheHeaderFn = fn(&Response<Body>) -> Option<HeaderValue>;

#[cfg(feature = "zerver")]
fn hourly_cache_layer() -> SetResponseHeaderLayer<CacheHeaderFn> {
    SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        hourly_public_cache as CacheHeaderFn,
    )
}

/// Routes that don't require authentication.
#[cfg(feature = "zerver")]
#[allow(clippy::expect_used)]
pub fn public_routes() -> Router<AppState> {
    // 5 req / 30s — tight limit, brute-force target
    let login_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(6))
            .burst_size(5)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 5 req / 1hr — rarely needed legitimately
    let register_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(720))
            .burst_size(5)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 20 req / 1min — clients refresh on cold start
    let refresh_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(3))
            .burst_size(20)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 5 req / 1hr per IP — password reset is rare
    let forgot_password_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(720))
            .burst_size(5)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 10 req / 1hr per IP — verify-email + reset-password
    let verify_reset_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(360))
            .burst_size(10)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 60 req / min per IP — card metadata is public and CF-cached, but a tight
    // limit guards the origin from someone bypassing CF to scrape directly.
    let public_card_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(1))
            .burst_size(60)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — public marketing aggregates. CF cache absorbs most
    // traffic; this guards origin if someone bypasses CF or warms many POPs.
    let public_marketing_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — min-version gate poll. ~30-byte payload hit once a
    // minute per app; modest cap guards origin without risking lockout noise.
    let public_client_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — public changelog. Fetched once per app launch and
    // Cloudflare edge-caches it, so origin traffic is tiny; this guards the
    // origin if someone bypasses CF or warms many POPs.
    let public_changelog_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 10 req / min per IP — pre-auth funnel events. A legitimate session
    // fires a handful ever (app opened, register viewed/submitted); this is
    // an unauthenticated write, so keep the row-spam ceiling low.
    let anonymous_event_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(6))
            .burst_size(10)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // Burst 2, then 1 req / min per IP — crash reports. A legitimate client
    // posts at most one per launch; this is an unauthenticated write that
    // inserts a row per call, so it gets the strictest limiter (the crash_id
    // upsert makes anything past it idempotent).
    let crash_report_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(60))
            .burst_size(2)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — public shared-deck reads. A page load fetches once
    // and CF may cache briefly; this guards origin against token scanning.
    let public_share_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
    );
    // 30 req / 2s per IP — health checks. Generous for frequent polling
    // (manual curls, uptime monitors) while capping unauthenticated floods,
    // notably /health/database which pings Postgres.
    let health_config = Arc::new(
        GovernorConfigBuilder::default()
            .period(Duration::from_secs(2))
            .burst_size(30)
            .key_extractor(CfConnectingIpKeyExtractor)
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
                .route("/database", get(are_server_and_database_running))
                .layer(GovernorLayer::new(health_config).error_handler(stable_rate_limit)),
        )
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/auth",
                    Router::new()
                        .route(
                            "/register",
                            post(register_user).layer(
                                GovernorLayer::new(register_config)
                                    .error_handler(stable_rate_limit),
                            ),
                        )
                        .route(
                            "/login",
                            post(authenticate_user).layer(
                                GovernorLayer::new(login_config).error_handler(stable_rate_limit),
                            ),
                        )
                        .route(
                            "/refresh",
                            post(refresh_session).layer(
                                GovernorLayer::new(refresh_config).error_handler(stable_rate_limit),
                            ),
                        )
                        .route(
                            "/verify-email",
                            post(verify_email).layer(
                                GovernorLayer::new(Arc::clone(&verify_reset_config))
                                    .error_handler(stable_rate_limit),
                            ),
                        )
                        .route(
                            "/forgot-password",
                            post(request_password_reset).layer(
                                GovernorLayer::new(forgot_password_config)
                                    .error_handler(stable_rate_limit),
                            ),
                        )
                        .route(
                            "/reset-password",
                            post(reset_password).layer(
                                GovernorLayer::new(verify_reset_config)
                                    .error_handler(stable_rate_limit),
                            ),
                        ),
                )
                .nest(
                    "/card",
                    Router::new()
                        .route("/{scryfall_data_id}", get(get_card))
                        .route("/featured-flavor", get(get_featured_flavor))
                        .route("/{oracle_id}/printings", get(get_printings))
                        .route("/artists", get(get_artists))
                        .route("/types", get(get_card_types))
                        .route("/keywords", get(get_keywords))
                        .route("/keyword-reminders", get(get_keyword_reminders))
                        .route("/roles", get(get_card_roles))
                        .route("/oracle-tags", get(get_oracle_tags))
                        .route("/oracle-words", get(get_oracle_words))
                        .route("/languages", get(get_languages))
                        .route("/sets", get(get_sets))
                        .layer(
                            GovernorLayer::new(public_card_config).error_handler(stable_rate_limit),
                        )
                        .layer(hourly_cache_layer()),
                )
                .nest(
                    "/marketing",
                    Router::new()
                        .route("/stats", get(get_public_metrics))
                        .layer(
                            GovernorLayer::new(public_marketing_config)
                                .error_handler(stable_rate_limit),
                        )
                        .layer(hourly_cache_layer()),
                )
                .nest(
                    "/client",
                    Router::new()
                        .route("/min-version", get(get_min_client_version))
                        .layer(
                            GovernorLayer::new(public_client_config)
                                .error_handler(stable_rate_limit),
                        ),
                )
                // Merged rather than routed directly: the cache layer hangs on
                // a wrapping Router because MethodRouter::layer can't infer
                // its error type when chained after the governor.
                .merge(
                    Router::new()
                        .route(
                            "/changelog",
                            get(get_changelog).layer(
                                GovernorLayer::new(public_changelog_config)
                                    .error_handler(stable_rate_limit),
                            ),
                        )
                        .layer(hourly_cache_layer()),
                )
                .nest(
                    "/metrics",
                    Router::new()
                        .route("/anonymous", post(record_anonymous_event))
                        .layer(
                            GovernorLayer::new(anonymous_event_config)
                                .error_handler(stable_rate_limit),
                        )
                        .merge(
                            Router::new()
                                .route("/crash", post(record_crash))
                                .layer(
                                    GovernorLayer::new(crash_report_config)
                                        .error_handler(stable_rate_limit),
                                )
                                // A crash report is ~2KB of truncated panic
                                // text; 4KB bounds this unauthed body hard.
                                .layer(DefaultBodyLimit::max(4096)),
                        ),
                )
                .nest(
                    "/share",
                    Router::new()
                        .route("/deck/{token}", get(get_shared_deck))
                        .layer(
                            GovernorLayer::new(public_share_config)
                                .error_handler(stable_rate_limit),
                        ),
                ),
        )
}

/// Routes that require `AuthenticatedUser` (JWT Bearer token).
#[cfg(feature = "zerver")]
#[allow(clippy::expect_used)]
pub fn private_routes(jwt_secret: JwtSecret) -> Router<AppState> {
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
                    Router::new().route("/logout", post(revoke_sessions)).route(
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
                            patch(change_password).layer(
                                GovernorLayer::new(Arc::clone(&sensitive_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route(
                            "/change-username",
                            patch(change_username).layer(
                                GovernorLayer::new(Arc::clone(&sensitive_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route(
                            "/change-email",
                            patch(change_email).layer(
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
                        .route(
                            "/preferences",
                            get(get_preferences).patch(update_preferences),
                        )
                        .route("/hint", patch(mark_hint_shown))
                        .route("/metrics", get(get_my_metrics))
                        .route(
                            "/commander-maybeboard",
                            get(get_commander_maybeboard).delete(clear_commander_maybeboard),
                        )
                        .route(
                            "/commander-maybeboard/{oracle_id}",
                            post(add_commander_maybeboard_card)
                                .delete(remove_commander_maybeboard_card),
                        ),
                )
                .nest(
                    "/card",
                    Router::new()
                        .route(
                            "/search",
                            post(search_cards).layer(
                                GovernorLayer::new(Arc::clone(&card_search_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        // Commander search shares the card-search budget: it's the
                        // same user behavior (typing in a select screen), just a
                        // dedicated serving path.
                        .route(
                            "/search/commanders",
                            post(search_commanders).layer(
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
                        .route("/tags", get(get_deck_tags))
                        .route("/{deck_id}/import/archidekt", post(import_archidekt_deck))
                        .route("/profile/{deck_id}", get(get_deck_profile))
                        .route(
                            "/{deck_id}",
                            get(get_deck).patch(update_deck_profile).delete(delete_deck),
                        )
                        .route("/{deck_id}/clone", post(clone_deck))
                        .route("/{deck_id}/share", post(share_deck).delete(unshare_deck))
                        .route(
                            "/{deck_id}/suppressions",
                            delete(clear_deck_suppressions).post(skip_deck_card),
                        )
                        .route(
                            "/{deck_id}/suppressions/{oracle_id}",
                            delete(unskip_deck_card),
                        )
                        .route("/{deck_id}/tokens", get(get_deck_tokens))
                        .nest(
                            "/{deck_id}/card",
                            Router::new()
                                .route("/", post(create_deck_card))
                                .route("/import", post(import_deck_cards))
                                .route("/search", post(search_deck_cards))
                                .route(
                                    "/{scryfall_data_id}",
                                    patch(patch_deck_card).delete(delete_deck_card),
                                ),
                        ),
                ),
        )
        .layer(GovernorLayer::new(private_config).error_handler(unauthorized_on_missing_key))
}

#[cfg(all(test, feature = "zerver"))]
mod tests {
    use super::rate_limit_copy;

    #[test]
    fn rate_limit_copy_is_stable_within_buckets() {
        // Short waits: one message for the whole window.
        assert_eq!(rate_limit_copy(1), rate_limit_copy(60));
        assert_eq!(rate_limit_copy(61), rate_limit_copy(300));
        // Long lockouts round up in 5-minute steps — 1784s and 1500s share
        // "about 30 minutes"; a fresh lockout in the next step differs.
        assert_eq!(
            rate_limit_copy(1784),
            "too many requests, try again in about 30 minutes"
        );
        assert_eq!(rate_limit_copy(1784), rate_limit_copy(1501));
        assert_ne!(rate_limit_copy(1500), rate_limit_copy(1501));
        // No raw seconds ever appear in the copy.
        assert!(!rate_limit_copy(1784).contains("1784"));
    }
}
