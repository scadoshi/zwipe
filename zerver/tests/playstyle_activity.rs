//! The Playstyle screen's activity: daily swipes and searches, swipe
//! outcomes, keep rate by card role and hard passes, read from the usage
//! tables the app's flushes fill. Also the summary route's first spelling,
//! served beside the new one until the client floor passes it.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use uuid::Uuid;
use zwipe_core::http::paths::{LEGACY_PLAY_STYLE_ROUTE, PLAYSTYLE_ACTIVITY_ROUTE, PLAYSTYLE_ROUTE};

/// Writes one day of usage `days_ago` UTC days back.
async fn seed_day(pool: &sqlx::PgPool, user: Uuid, days_ago: i32, right: i32, searches: i32) {
    sqlx::query(
        "INSERT INTO user_daily_activity (user_id, day, swipes_right, swipes_left, searches)
         VALUES ($1, (now() AT TIME ZONE 'UTC')::date - $2, $3, 1, $4)",
    )
    .bind(user)
    .bind(days_ago)
    .bind(right)
    .bind(searches)
    .execute(pool)
    .await
    .unwrap();
}

/// Writes one card's signal under one commander.
async fn seed_signal(
    pool: &sqlx::PgPool,
    user: Uuid,
    commander: Uuid,
    card: Uuid,
    (added, skipped, maybed): (i64, i64, i64),
) {
    sqlx::query(
        "INSERT INTO user_card_signal (user_id, commander_oracle_id, card_oracle_id, shown, added, skipped, maybed)
         VALUES ($1, $2, $3, $4 + $5 + $6, $4, $5, $6)",
    )
    .bind(user)
    .bind(commander)
    .bind(card)
    .bind(added)
    .bind(skipped)
    .bind(maybed)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn activity_needs_a_session(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.get(PLAYSTYLE_ACTIVITY_ROUTE, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn the_first_spelling_still_answers_with_the_summary(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, uid) = app.register("spelling").await;
    app.verify_email(&uid).await;

    let (status, new) = app.get(PLAYSTYLE_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{new}");
    let (status, old) = app.get(LEGACY_PLAY_STYLE_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{old}");
    assert_eq!(new, old, "both spellings serve the same summary");
}

#[sqlx::test]
async fn a_new_account_has_no_activity(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (token, uid) = app.register("quiet").await;
    app.verify_email(&uid).await;

    let (status, body) = app.get(PLAYSTYLE_ACTIVITY_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["days"].as_array().unwrap().len(), 0, "{body}");
    assert_eq!(body["outcomes"]["added"], 0, "{body}");
    assert_eq!(body["roles"].as_array().unwrap().len(), 0, "{body}");
    assert_eq!(body["passes"].as_array().unwrap().len(), 0, "{body}");
}

#[sqlx::test]
async fn activity_reads_the_year_the_outcomes_the_roles_and_the_passes(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("busy").await;
    app.verify_email(&uid).await;
    let user = Uuid::parse_str(&uid).unwrap();

    // Two days inside the year and one past it.
    seed_day(&pool, user, 0, 9, 2).await;
    seed_day(&pool, user, 3, 4, 0).await;
    seed_day(&pool, user, 400, 50, 5).await;

    sqlx::query(
        "INSERT INTO user_week_signal (user_id, week_start, added, skipped, maybed, removed)
         VALUES ($1, '2026-10-05', 3, 7, 2, 1), ($1, '2026-10-12', 1, 1, 0, 0)",
    )
    .bind(user)
    .execute(&pool)
    .await
    .unwrap();

    let ramp = card("Steady Ramp").categories(&["ramp"]);
    let removal = card("Clean Removal").categories(&["removal"]);
    let rare = card("Rarely Seen").categories(&["draw"]);
    let passed = card("Never Wanted").categories(&["removal"]);
    let once = card("Skipped Once").categories(&["ramp"]);
    let ids: Vec<Uuid> = [&ramp, &removal, &rare, &passed, &once]
        .iter()
        .map(|c| c.oracle_id().unwrap())
        .collect();
    seed_cards(&pool, &[ramp, removal, rare, passed, once]).await;
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    seed_signal(&pool, user, a, ids[0], (8, 2, 0)).await;
    seed_signal(&pool, user, a, ids[1], (1, 4, 1)).await;
    seed_signal(&pool, user, a, ids[2], (1, 1, 0)).await;
    seed_signal(&pool, user, a, ids[3], (0, 3, 0)).await;
    seed_signal(&pool, user, b, ids[3], (0, 2, 0)).await;
    seed_signal(&pool, user, a, ids[4], (0, 1, 0)).await;

    let (status, body) = app.get(PLAYSTYLE_ACTIVITY_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let days = body["days"].as_array().unwrap();
    assert_eq!(days.len(), 2, "the day past the year is left out: {body}");
    assert_eq!(
        days[0]["swipes"], 5,
        "oldest first, every direction: {body}"
    );
    assert_eq!(days[1]["swipes"], 10, "{body}");
    assert_eq!(days[1]["searches"], 2, "{body}");
    assert_eq!(
        days[1]["day"], body["through"],
        "today closes the year: {body}"
    );

    assert_eq!(body["outcomes"]["added"], 4, "{body}");
    assert_eq!(body["outcomes"]["skipped"], 8, "{body}");
    assert_eq!(body["outcomes"]["maybed"], 2, "{body}");
    assert_eq!(body["outcomes"]["removed"], 1, "{body}");

    // ramp 8 kept and 3 passed, 11 seen; removal 1 and 9, 10 seen; draw 2
    // seen, under the floor.
    let roles: Vec<(&str, u64, u64)> = body["roles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["role"].as_str().unwrap(),
                r["kept"].as_u64().unwrap(),
                r["passed"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(roles, [("ramp", 8, 3), ("removal", 1, 9)], "{body}");

    let passes = body["passes"].as_array().unwrap();
    assert_eq!(
        passes.len(),
        1,
        "kept, saved and once-skipped cards are no hard pass: {body}"
    );
    assert_eq!(
        passes[0]["card"]["scryfall_data"]["name"], "Never Wanted",
        "{body}"
    );
    assert_eq!(passes[0]["skipped"], 5, "{body}");
    assert_eq!(passes[0]["commanders"], 2, "{body}");
}

/// Writes every player's swipes on one card under one commander, the pooled
/// table hard passes are weighed against.
async fn seed_pooled(
    pool: &sqlx::PgPool,
    commander: Uuid,
    card: Uuid,
    (added, skipped): (i64, i64),
) {
    sqlx::query(
        "INSERT INTO commander_card_signal (commander_oracle_id, card_oracle_id, shown, added, skipped)
         VALUES ($1, $2, $3 + $4, $3, $4)",
    )
    .bind(commander)
    .bind(card)
    .bind(added)
    .bind(skipped)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn hard_passes_weigh_skips_against_other_players(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("picky").await;
    app.verify_email(&uid).await;
    let user = Uuid::parse_str(&uid).unwrap();

    let common = card("Everyone Passes");
    let kept = card("Others Keep");
    let (common_id, kept_id) = (common.oracle_id().unwrap(), kept.oracle_id().unwrap());
    seed_cards(&pool, &[common, kept]).await;
    let (mine, theirs) = (Uuid::new_v4(), Uuid::new_v4());

    // The user skips the common card more often than the other.
    seed_signal(&pool, user, mine, common_id, (0, 6, 0)).await;
    seed_signal(&pool, user, mine, kept_id, (0, 2, 0)).await;
    // The pool carries the user's skips and everyone else's: the others skip
    // the common card 98 times in 100 and keep the other 4 times in 5.
    seed_pooled(&pool, mine, common_id, (0, 6)).await;
    seed_pooled(&pool, theirs, common_id, (2, 98)).await;
    seed_pooled(&pool, mine, kept_id, (0, 2)).await;
    seed_pooled(&pool, theirs, kept_id, (40, 10)).await;

    let (status, body) = app.get(PLAYSTYLE_ACTIVITY_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let names: Vec<&str> = body["passes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["card"]["scryfall_data"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["Others Keep"],
        "a card everyone skips needs many more skips than six: {body}"
    );
}
