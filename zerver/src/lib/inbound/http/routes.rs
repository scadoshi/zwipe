//! Route definitions mapping paths to handlers.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode, header};
use axum::routing::{delete, get, patch, post};
use tower_governor::GovernorLayer;
use tower_governor::errors::GovernorError;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::domain::auth::models::access_token::JwtSecret;
use crate::inbound::http::AppState;
use crate::inbound::http::family_limit::FamilyLimitLayer;
use crate::inbound::http::handlers::{
    auth, card, changelog, client, deck, deck_card, health, metrics, user,
};
use crate::inbound::http::limiter_purge::{LIMITER_PURGE_EVERY, purging};
use crate::inbound::http::middleware::UserIdKeyExtractor;
use crate::inbound::http::rate_limits::RateLimits;

/// Largest crash report body accepted, in bytes. A report is about 2 KB of
/// truncated panic text, and the route takes it unauthenticated.
const CRASH_REPORT_BODY_LIMIT: usize = 4096;

/// Largest card lookup body accepted, in bytes. The largest valid lookup, a 100-character name
/// and fifteen short chip words, is about 1 KB, and the route takes it signed out.
const CARD_LOOKUP_BODY_LIMIT: usize = 4096;

/// Largest decklist import body accepted, in bytes. A 500-card list pasted
/// from any deck site is about 15 KB.
const IMPORT_BODY_LIMIT: usize = 64 * 1024;

/// Rate-limit error handler for the routes keyed by user id
/// (`UserIdKeyExtractor`).
///
/// On those routes the governor layer runs *before* the `AuthenticatedUser`
/// extractor, and it calls the key extractor to bucket the request. When the
/// request has no valid Bearer token the extractor returns
/// `GovernorError::UnableToExtractKey`, which `tower_governor`'s default maps
/// to a **500**.
///
/// This remaps that one case to **401 Unauthorized** (the honest status for
/// "you didn't authenticate"). `TooManyRequests` gets the stable 429 body
/// (see [`too_many_requests_response`]); anything else delegates to the
/// library default.
///
/// Only attached to the user-id-keyed limiters; the public, IP-keyed limiters
/// can always extract a key and use [`stable_rate_limit`] instead.
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
        other @ GovernorError::Other { .. } => other.into_response().map(Body::from),
    }
}

/// Rate-limit error handler for the public, IP-keyed limiters: the stable 429
/// body for `TooManyRequests`, library default for everything else (their key
/// extraction never legitimately fails, so no 401 remap here).
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "tower_governor's error is a foreign enum: every other error keeps the library's response"
)]
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
/// `tower_governor`'s default body interpolates the live countdown ("Too Many
/// Requests! Wait for 1784s"); the client shows that text in a toast AND uses
/// it in the error-report dedupe key, so the countdown would mint a fresh
/// `client_errors` row for every second of a lockout. Bucketed copy keeps the
/// message stable across a lockout; machines that want precision read the
/// header.
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
#[expect(
    clippy::arithmetic_side_effects,
    reason = "u64::MAX / 300 * 5 still fits in u64"
)]
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

/// `Cache-Control: public, max-age=3600` for successful responses on the
/// public read endpoints (card catalogs, marketing stats), so the
/// CF edge (whose cache rules respect origin TTL) holds one copy per hour
/// instead of falling back to its daily default. Two handlers set their own,
/// which `if_not_present` lets win: featured-flavor counts down to the hour
/// its content flips on, and the marketing stats keep the edge within five
/// minutes of the origin.
///
/// Success-only, so a 429 or error body is never browser-cacheable.
fn hourly_public_cache(response: &Response<Body>) -> Option<HeaderValue> {
    response
        .status()
        .is_success()
        .then(|| HeaderValue::from_static("public, max-age=3600"))
}

type CacheHeaderFn = fn(&Response<Body>) -> Option<HeaderValue>;

fn hourly_cache_layer() -> SetResponseHeaderLayer<CacheHeaderFn> {
    let make_header: CacheHeaderFn = hourly_public_cache;
    SetResponseHeaderLayer::if_not_present(header::CACHE_CONTROL, make_header)
}

/// `Cache-Control: private, no-store` on every authenticated response, so no
/// edge or browser cache can hold one user's data and hand it to another,
/// whatever cache rules are configured in front of the origin.
fn private_no_store_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    )
}

/// Routes that don't require authentication.
///
/// # Panics
///
/// Never in practice: a limit only fails to build on a zero period or burst
/// ([`FamilyLimitLayer::new`]), and every one here is a non-zero constant.
#[expect(
    clippy::too_many_lines,
    reason = "one router tree: each public limiter, then the routes it guards"
)]
pub fn public_routes(limits: &RateLimits) -> Router<AppState> {
    let login_limit = FamilyLimitLayer::new(limits.login, stable_rate_limit);
    let register_limit = FamilyLimitLayer::new(limits.register, stable_rate_limit);
    let refresh_limit = FamilyLimitLayer::new(limits.refresh, stable_rate_limit);
    let forgot_password_limit = FamilyLimitLayer::new(limits.forgot_password, stable_rate_limit);
    let verify_reset_limit = FamilyLimitLayer::new(limits.verify_reset, stable_rate_limit);
    let public_card_limit = FamilyLimitLayer::new(limits.public_card, stable_rate_limit);
    let public_examples_limit = FamilyLimitLayer::new(limits.public_examples, stable_rate_limit);
    let public_lookup_limit = FamilyLimitLayer::new(limits.public_lookup, stable_rate_limit);
    let public_marketing_limit = FamilyLimitLayer::new(limits.public_marketing, stable_rate_limit);
    let public_client_limit = FamilyLimitLayer::new(limits.public_client, stable_rate_limit);
    let public_changelog_limit = FamilyLimitLayer::new(limits.public_changelog, stable_rate_limit);
    let anonymous_event_limit = FamilyLimitLayer::new(limits.anonymous_event, stable_rate_limit);
    let crash_report_limit = FamilyLimitLayer::new(limits.crash_report, stable_rate_limit);
    let public_share_limit = FamilyLimitLayer::new(limits.public_share, stable_rate_limit);
    let share_preview_limit = FamilyLimitLayer::new(limits.share_preview, stable_rate_limit);
    let health_limit = FamilyLimitLayer::new(limits.health, stable_rate_limit);

    Router::new()
        .route("/", get(health::root))
        .nest(
            "/health",
            Router::new()
                .route("/", get(health::are_server_and_database_running))
                .route("/server", get(health::is_server_running))
                .route("/database", get(health::are_server_and_database_running))
                .layer(health_limit),
        )
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/auth",
                    Router::new()
                        .route("/register", post(auth::register_user).layer(register_limit))
                        .route(
                            "/login",
                            post(auth::authenticate_user).layer(login_limit.clone()),
                        )
                        .route(
                            "/refresh",
                            post(auth::refresh_session).layer(refresh_limit.clone()),
                        )
                        // The browser's session, its refresh token in an
                        // HttpOnly cookie. Shares the app routes' limits.
                        .route("/cookie/login", post(auth::cookie_login).layer(login_limit))
                        .route(
                            "/cookie/refresh",
                            post(auth::cookie_refresh).layer(refresh_limit.clone()),
                        )
                        // Public: the cookie alone logs the browser out when its
                        // access token can't be had, so no bearer is required.
                        .route(
                            "/cookie/logout",
                            post(auth::cookie_logout).layer(refresh_limit),
                        )
                        .route(
                            "/verify-email",
                            post(auth::verify_email).layer(verify_reset_limit.clone()),
                        )
                        .route(
                            "/confirm-email-change",
                            post(auth::confirm_email_change).layer(verify_reset_limit.clone()),
                        )
                        .route(
                            "/forgot-password",
                            post(auth::request_password_reset).layer(forgot_password_limit),
                        )
                        .route(
                            "/reset-password/check",
                            post(auth::check_reset_token).layer(verify_reset_limit.clone()),
                        )
                        .route(
                            "/reset-password",
                            post(auth::reset_password).layer(verify_reset_limit),
                        ),
                )
                .nest(
                    "/card",
                    Router::new()
                        .route("/{scryfall_data_id}", get(card::get_card))
                        .route("/named/{name_slug}", get(card::get_named_cards))
                        .route("/featured-flavor", get(card::get_featured_flavor))
                        .route("/{oracle_id}/printings", get(card::get_printings))
                        .route("/artists", get(card::get_artists))
                        .route("/types", get(card::get_card_types))
                        .route("/keywords", get(card::get_keywords))
                        .route("/keyword-reminders", get(card::get_keyword_reminders))
                        .route("/roles", get(card::get_card_roles))
                        .route("/ub-franchises", get(card::get_ub_franchises))
                        .route("/oracle-tags", get(card::get_oracle_tags))
                        .route("/oracle-words", get(card::get_oracle_words))
                        .route("/languages", get(card::get_languages))
                        .route("/sets", get(card::get_sets))
                        .layer(public_card_limit)
                        .layer(hourly_cache_layer()),
                )
                .nest(
                    "/card/oracle-tag",
                    Router::new()
                        .route("/{slug}/examples", get(card::get_oracle_tag_examples))
                        .layer(public_examples_limit)
                        .layer(hourly_cache_layer()),
                )
                .merge(
                    Router::new()
                        .route("/card/lookup", post(card::lookup_cards))
                        .layer(public_lookup_limit)
                        .layer(DefaultBodyLimit::max(CARD_LOOKUP_BODY_LIMIT))
                        // A signed-in answer follows the user's Universes Beyond setting, so no
                        // answer, a refusal included, is kept by a cache.
                        .layer(private_no_store_layer()),
                )
                .nest(
                    "/marketing",
                    Router::new()
                        .route("/stats", get(metrics::get_public_metrics))
                        .layer(public_marketing_limit)
                        .layer(hourly_cache_layer()),
                )
                .nest(
                    "/client",
                    Router::new()
                        .route("/min-version", get(client::get_min_client_version))
                        .layer(public_client_limit),
                )
                // The handler sets its own Cache-Control (no-cache plus an
                // ETag), so the hourly layer stays off this route.
                .route(
                    "/changelog",
                    get(changelog::get_changelog).layer(public_changelog_limit),
                )
                .nest(
                    "/metrics",
                    Router::new()
                        .route("/anonymous", post(metrics::record_anonymous_event))
                        .layer(anonymous_event_limit)
                        .merge(
                            Router::new()
                                .route("/crash", post(metrics::record_crash))
                                .layer(crash_report_limit)
                                .layer(DefaultBodyLimit::max(CRASH_REPORT_BODY_LIMIT)),
                        ),
                )
                .nest(
                    "/share",
                    Router::new()
                        .route("/deck/{token}", get(deck::get_shared_deck))
                        .route(
                            "/deck/{token}/preview.png",
                            get(deck::get_shared_deck_preview).layer(share_preview_limit),
                        )
                        .layer(public_share_limit),
                ),
        )
}

/// Routes that require `AuthenticatedUser` (JWT Bearer token).
///
/// # Panics
///
/// Never in practice: a governor config only fails to build on a zero period
/// or burst, and every one here is a non-zero constant.
#[expect(
    clippy::expect_used,
    reason = "every governor config has a non-zero period and burst, so finish never returns None"
)]
#[expect(
    clippy::too_many_lines,
    reason = "one router tree: each per-user limiter, then the routes it guards"
)]
pub fn private_routes(jwt_secret: JwtSecret, limits: &RateLimits) -> Router<AppState> {
    let private_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.private.period)
            .burst_size(limits.private.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );
    let sensitive_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.sensitive.period)
            .burst_size(limits.sensitive.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );
    let card_search_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.card_search.period)
            .burst_size(limits.card_search.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );
    let resend_verification_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.resend_verification.period)
            .burst_size(limits.resend_verification.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );
    let resend_email_change_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.resend_email_change.period)
            .burst_size(limits.resend_email_change.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );
    let metrics_usage_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.metrics_usage.period)
            .burst_size(limits.metrics_usage.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret.clone()))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );
    let archidekt_import_config = purging(
        GovernorConfigBuilder::default()
            .period(limits.archidekt_import.period)
            .burst_size(limits.archidekt_import.burst)
            .key_extractor(UserIdKeyExtractor::new(jwt_secret))
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    );

    Router::new()
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/auth",
                    Router::new()
                        .route("/logout", post(auth::revoke_sessions))
                        .route(
                            "/resend-verification",
                            post(auth::resend_verification).layer(
                                GovernorLayer::new(resend_verification_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route("/cancel-email-change", post(auth::cancel_email_change))
                        .route(
                            "/resend-email-change",
                            post(auth::resend_email_change).layer(
                                GovernorLayer::new(resend_email_change_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        ),
                )
                .nest(
                    "/user",
                    Router::new()
                        .route("/", get(user::get_user))
                        .route(
                            "/change-password",
                            patch(auth::change_password).layer(
                                GovernorLayer::new(Arc::clone(&sensitive_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route(
                            "/change-username",
                            patch(auth::change_username).layer(
                                GovernorLayer::new(Arc::clone(&sensitive_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route(
                            "/change-email",
                            patch(auth::change_email).layer(
                                GovernorLayer::new(Arc::clone(&sensitive_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route(
                            "/delete-user",
                            delete(auth::delete_user).layer(
                                GovernorLayer::new(sensitive_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route(
                            "/settings",
                            get(user::fetch_settings).patch(user::update_settings),
                        )
                        .route("/hint", patch(user::mark_hint_shown))
                        .route("/metrics", get(metrics::get_my_metrics))
                        .route("/activity", get(deck::get_activity))
                        .route(
                            "/commander-maybeboard",
                            get(user::get_commander_maybeboard)
                                .delete(user::clear_commander_maybeboard),
                        )
                        .route(
                            "/commander-maybeboard/{oracle_id}",
                            post(user::add_commander_maybeboard_card)
                                .delete(user::remove_commander_maybeboard_card),
                        ),
                )
                .nest(
                    "/commander",
                    Router::new()
                        .route("/suggestions", get(deck::get_commander_suggestions))
                        .route(
                            "/suggestions/dismissals",
                            post(deck::dismiss_commander).delete(deck::clear_commander_dismissals),
                        )
                        .route(
                            "/suggestions/dismissals/{oracle_id}",
                            delete(deck::undismiss_commander),
                        ),
                )
                .nest(
                    "/card",
                    Router::new()
                        .route(
                            "/search",
                            post(card::search_cards).layer(
                                GovernorLayer::new(Arc::clone(&card_search_config))
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        // Commander search shares the card-search budget: it's the
                        // same user behavior (typing in a select screen), just a
                        // dedicated serving path.
                        .route(
                            "/search/commanders",
                            post(card::search_commanders).layer(
                                GovernorLayer::new(card_search_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        ),
                )
                .nest(
                    "/metrics",
                    Router::new().route(
                        "/usage",
                        post(metrics::record_usage).layer(
                            GovernorLayer::new(metrics_usage_config)
                                .error_handler(unauthorized_on_missing_key),
                        ),
                    ),
                )
                .nest(
                    "/deck",
                    Router::new()
                        .route(
                            "/",
                            get(deck::get_deck_profiles).post(deck::create_deck_profile),
                        )
                        .route("/tags", get(deck::get_deck_tags))
                        .route("/playstyle", get(deck::get_play_style_summary))
                        .route(
                            "/{deck_id}/import/archidekt",
                            post(deck::import_archidekt_deck).layer(
                                GovernorLayer::new(archidekt_import_config)
                                    .error_handler(unauthorized_on_missing_key),
                            ),
                        )
                        .route("/profile/{deck_id}", get(deck::get_deck_profile))
                        .route(
                            "/{deck_id}",
                            get(deck::get_deck)
                                .patch(deck::update_deck_profile)
                                .delete(deck::delete_deck),
                        )
                        .route("/{deck_id}/clone", post(deck::clone_deck))
                        .route(
                            "/{deck_id}/share",
                            post(deck::share_deck).delete(deck::unshare_deck),
                        )
                        .route(
                            "/{deck_id}/suppressions",
                            delete(deck::clear_deck_suppressions).post(deck::skip_deck_card),
                        )
                        .route(
                            "/{deck_id}/suppressions/{oracle_id}",
                            delete(deck::unskip_deck_card),
                        )
                        .route("/{deck_id}/tokens", get(deck::get_deck_tokens))
                        .nest(
                            "/{deck_id}/card",
                            Router::new()
                                .route("/", post(deck_card::create_deck_card))
                                .route(
                                    "/import",
                                    post(deck_card::import_deck_cards)
                                        .layer(DefaultBodyLimit::max(IMPORT_BODY_LIMIT)),
                                )
                                .route("/search", post(deck::search_deck_cards))
                                .route(
                                    "/{scryfall_data_id}",
                                    patch(deck_card::patch_deck_card)
                                        .delete(deck_card::delete_deck_card),
                                ),
                        ),
                ),
        )
        .layer(GovernorLayer::new(private_config).error_handler(unauthorized_on_missing_key))
        .layer(private_no_store_layer())
}

#[cfg(test)]
mod tests {
    use super::rate_limit_copy;

    #[test]
    fn rate_limit_copy_is_stable_within_buckets() {
        // Short waits: one message for the whole window.
        assert_eq!(rate_limit_copy(1), rate_limit_copy(60));
        assert_eq!(rate_limit_copy(61), rate_limit_copy(300));
        // Long lockouts round up in 5-minute steps: 1784s and 1500s share
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
