//! Suggested commanders: the user's commander decks scored against seeded
//! `commander_tag_profiles` rows, with the reasons behind each pick, the
//! per-deck matching, the color identity cap, the exclusions, and the
//! week-long dismissals.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use axum::http::StatusCode;
use common::{TestApp, card, seed_cards};
use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::http::paths::{
    COMMANDER_DISMISSALS_ROUTE, COMMANDER_SUGGESTIONS_ROUTE, DECK_ROUTE,
    add_commander_maybeboard_card_route, get_deck_route, undismiss_commander_route,
};

/// Writes one commander's precomputed tag profile, as the synergy worker does.
async fn seed_profile(pool: &sqlx::PgPool, oracle_id: Uuid, profile: Value) {
    sqlx::query("INSERT INTO commander_tag_profiles (oracle_id, profile) VALUES ($1, $2)")
        .bind(oracle_id)
        .bind(sqlx::types::Json(profile))
        .execute(pool)
        .await
        .unwrap();
}

/// Writes the per-oracle tag rows the deck tag profile reads.
async fn seed_oracle_tags(pool: &sqlx::PgPool, rows: &[(Uuid, &str)]) {
    for (oracle_id, tag) in rows {
        sqlx::query("INSERT INTO card_oracle_tags (oracle_id, oracle_tag) VALUES ($1, $2)")
            .bind(oracle_id)
            .bind(tag)
            .execute(pool)
            .await
            .unwrap();
    }
}

/// Creates a Commander deck led by `commander_scryfall_id` and puts
/// `mainboard` (scryfall id, oracle id) on its mainboard.
async fn build_commander_deck(
    app: &TestApp,
    token: &str,
    name: &str,
    commander_scryfall_id: Uuid,
    mainboard: &[(Uuid, Uuid)],
) -> Uuid {
    let (status, deck) = app
        .post(
            DECK_ROUTE,
            json!({ "name": name, "format": "commander" }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let deck_id: Uuid = deck["id"].as_str().unwrap().parse().unwrap();
    let (status, updated) = app
        .patch(
            &get_deck_route(deck_id),
            json!({ "commander_id": commander_scryfall_id.to_string() }),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set commander: {updated}");
    for (scryfall_id, oracle_id) in mainboard {
        sqlx::query(
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board)
             VALUES ($1, $2, $3, 1, 'deck')",
        )
        .bind(deck_id)
        .bind(scryfall_id)
        .bind(oracle_id)
        .execute(&app.pool)
        .await
        .unwrap();
    }
    deck_id
}

async fn suggestions(app: &TestApp, token: &str, limit: u32) -> Value {
    let (status, body) = app
        .get(
            &format!("{COMMANDER_SUGGESTIONS_ROUTE}?limit={limit}"),
            Some(token),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "suggestions: {body}");
    body
}

fn names(body: &Value) -> Vec<String> {
    body["suggestions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            s["card"]["scryfall_data"]["name"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

/// A goblin deck plus four profiled candidates: one sharing its tags, one
/// sharing none, the deck's own commander and a maybeboarded one. Returns
/// the token and the two candidates' oracle ids (match, stranger).
async fn goblin_user(app: &TestApp) -> (String, Uuid, Uuid) {
    let (token, uid) = app.register("gobbo").await;
    app.verify_email(&uid).await;

    let commander = card("Goblin Boss")
        .mono("R")
        .type_line("Legendary Creature — Goblin")
        .commander_legal()
        .oracle_tags(&["typal-goblin"]);
    let goblin_a = card("Goblin A").mono("R");
    let goblin_b = card("Goblin B").mono("R");
    let matching = card("Krenko Match")
        .mono("R")
        .type_line("Legendary Creature — Goblin")
        .commander_legal();
    let stranger = card("Lifegain Lord")
        .mono("W")
        .type_line("Legendary Creature — Angel")
        .commander_legal();
    let maybe = card("Maybe Commander")
        .mono("R")
        .type_line("Legendary Creature — Goblin")
        .commander_legal();

    let cmd_sid = commander.id();
    let cmd_oid = commander.oracle_id().unwrap();
    let a = (goblin_a.id(), goblin_a.oracle_id().unwrap());
    let b = (goblin_b.id(), goblin_b.oracle_id().unwrap());
    let match_oid = matching.oracle_id().unwrap();
    let stranger_oid = stranger.oracle_id().unwrap();
    let maybe_oid = maybe.oracle_id().unwrap();

    seed_cards(
        &app.pool,
        &[commander, goblin_a, goblin_b, matching, stranger, maybe],
    )
    .await;
    seed_oracle_tags(
        &app.pool,
        &[
            (cmd_oid, "typal-goblin"),
            (a.1, "typal-goblin"),
            (a.1, "sacrifice-outlet"),
            (b.1, "typal-goblin"),
            // A tag on every tagged card carries no information, so a card
            // outside the deck keeps the goblin tags informative.
            (stranger_oid, "lifegain"),
        ],
    )
    .await;
    // sacrifice-outlet is a root of the sacrifice role, so it groups under it
    // once the catalog carries the row; typal-goblin falls under no role.
    sqlx::query(
        "INSERT INTO oracle_tags (id, slug, label) VALUES \
         ($1, 'sacrifice-outlet', 'Sacrifice outlet'), ($2, 'typal-goblin', 'Goblin typal')",
    )
    .bind(Uuid::from_u128(0x7a5))
    .bind(Uuid::from_u128(0x7a6))
    .execute(&app.pool)
    .await
    .unwrap();

    // The rare sacrifice tag outweighs the goblin tag in the user's profile,
    // so the candidate's own lean decides which tag comes first.
    seed_profile(
        &app.pool,
        match_oid,
        json!({ "typal-goblin": 0.8, "sacrifice-outlet": 0.2 }),
    )
    .await;
    seed_profile(&app.pool, stranger_oid, json!({ "lifegain": 1.0 })).await;
    seed_profile(&app.pool, cmd_oid, json!({ "typal-goblin": 1.0 })).await;
    seed_profile(&app.pool, maybe_oid, json!({ "typal-goblin": 1.0 })).await;

    build_commander_deck(app, &token, "Gobbos", cmd_sid, &[a, b]).await;
    let (status, body) = app
        .post(
            &add_commander_maybeboard_card_route(maybe_oid),
            json!({}),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "maybeboard: {body}");

    (token, match_oid, stranger_oid)
}

#[sqlx::test]
async fn cold_start_is_empty_with_zero_decks(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("coldstart").await;
    app.verify_email(&uid).await;

    let candidate = card("Any Commander").mono("G").commander_legal();
    let candidate_oid = candidate.oracle_id().unwrap();
    seed_cards(&pool, &[candidate]).await;
    seed_profile(&pool, candidate_oid, json!({ "ramp": 1.0 })).await;

    let body = suggestions(&app, &token, 10).await;
    assert_eq!(
        body,
        json!({ "suggestions": [], "deck_count": 0, "profile": null })
    );

    // A deck without a commander feeds nothing either.
    let (status, deck) = app
        .post(DECK_ROUTE, json!({ "name": "Headless" }), Some(&token))
        .await;
    assert_eq!(status, StatusCode::CREATED, "create: {deck}");
    let body = suggestions(&app, &token, 10).await;
    assert_eq!(body["deck_count"], 0);
    assert!(names(&body).is_empty());
    assert_eq!(body["profile"], Value::Null);
}

#[sqlx::test]
async fn shared_tags_rank_first_with_reasons_and_exclusions(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, _, _) = goblin_user(&app).await;

    let body = suggestions(&app, &token, 10).await;
    assert_eq!(body["deck_count"], 1);
    assert_eq!(
        names(&body),
        vec!["Krenko Match", "Lifegain Lord"],
        "the deck's own commander and the maybeboarded one are excluded: {body}"
    );

    let reasons = &body["suggestions"][0]["reasons"];
    assert_eq!(
        reasons["tags"],
        json!([
            { "slug": "typal-goblin", "role": null },
            { "slug": "sacrifice-outlet", "role": "sacrifice" },
        ]),
        "shared tags strongest first, each under the card display's role or none"
    );
    assert_eq!(reasons["colors"], json!([{ "color": "R", "decks": 1 }]));

    let stranger = &body["suggestions"][1]["reasons"];
    assert_eq!(stranger["tags"], json!([]), "no shared tags");
    assert_eq!(stranger["colors"], json!([]), "white decks: none");

    // The profile summarizes the one red deck: the rare sacrifice tag
    // outweighs the goblin tag in it, each carried by that one deck.
    assert_eq!(
        body["profile"],
        json!({
            "deck_count": 1,
            "colors": [{ "color": "R", "decks": 1 }],
            "top_identity": { "colors": ["R"], "decks": 1 },
            "tags": [
                { "slug": "sacrifice-outlet", "role": "sacrifice", "decks": 1 },
                { "slug": "typal-goblin", "role": null, "decks": 1 },
            ],
        }),
        "profile: {body}"
    );
}

#[sqlx::test]
async fn dismissals_hide_undo_clear_and_expire(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, match_oid, stranger_oid) = goblin_user(&app).await;

    let (status, body) = app
        .post(
            COMMANDER_DISMISSALS_ROUTE,
            json!({ "oracle_id": match_oid }),
            Some(&token),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "dismiss: {body}");
    assert_eq!(
        names(&suggestions(&app, &token, 10).await),
        vec!["Lifegain Lord"],
        "a dismissed commander drops out"
    );

    let (status, _) = app
        .delete(&undismiss_commander_route(match_oid), Some(&token))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "undo");
    assert_eq!(
        names(&suggestions(&app, &token, 10).await),
        vec!["Krenko Match", "Lifegain Lord"],
        "undo restores it"
    );

    for oracle_id in [match_oid, stranger_oid] {
        let (status, _) = app
            .post(
                COMMANDER_DISMISSALS_ROUTE,
                json!({ "oracle_id": oracle_id }),
                Some(&token),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert!(names(&suggestions(&app, &token, 10).await).is_empty());
    let (status, body) = app.delete(COMMANDER_DISMISSALS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "clear: {body}");
    assert_eq!(body, json!({ "cleared": 2 }));
    assert_eq!(
        names(&suggestions(&app, &token, 10).await),
        vec!["Krenko Match", "Lifegain Lord"],
        "clear restores all"
    );

    // A dismissal older than the week is ignored.
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = 'gobbo'")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO user_commander_dismissals (user_id, oracle_id, dismissed_at)
         VALUES ($1, $2, now() - interval '8 days')",
    )
    .bind(user_id)
    .bind(match_oid)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        names(&suggestions(&app, &token, 10).await),
        vec!["Krenko Match", "Lifegain Lord"],
        "an aged dismissal is ignored"
    );
}

/// Writes one legal commander with the given color identity and profile.
async fn seed_candidate(pool: &sqlx::PgPool, name: &str, colors: &str, profile: Value) {
    let candidate = card(name).mono(colors).commander_legal();
    let oid = candidate.oracle_id().unwrap();
    seed_cards(pool, &[candidate]).await;
    seed_profile(pool, oid, profile).await;
}

#[sqlx::test]
async fn color_identity_matches_as_a_set_not_an_average(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("jeskai").await;
    app.verify_email(&uid).await;

    // Two Jeskai decks and a Boros one, every mainboard card a token maker.
    let jeskai_a = card("Jeskai Lead A").mono("WUR").commander_legal();
    let jeskai_b = card("Jeskai Lead B").mono("WUR").commander_legal();
    let boros = card("Boros Lead").mono("WR").commander_legal();
    let tokens = card("Token Maker").mono("W");
    let other = card("Untracked Burn").mono("R");
    let tokens_pair = (tokens.id(), tokens.oracle_id().unwrap());
    let leads = [jeskai_a.id(), jeskai_b.id(), boros.id()];
    seed_oracle_tags(
        &pool,
        &[
            (tokens_pair.1, "tokens"),
            // Keeps "tokens" short of every tagged card, so it carries weight.
            (other.oracle_id().unwrap(), "burn"),
        ],
    )
    .await;
    seed_cards(&pool, &[jeskai_a, jeskai_b, boros, tokens, other]).await;
    for (i, lead) in leads.into_iter().enumerate() {
        build_commander_deck(&app, &token, &format!("Deck {i}"), lead, &[tokens_pair]).await;
    }

    // Same tags; the Jeskai candidate matches a Jeskai deck outright while
    // mono-blue covers a third of it. Averaging the user's color shares would
    // have put blue first, since every deck has blue's partners.
    seed_candidate(&pool, "Jeskai Candidate", "WUR", json!({ "tokens": 1.0 })).await;
    seed_candidate(&pool, "Blue Candidate", "U", json!({ "tokens": 1.0 })).await;

    let body = suggestions(&app, &token, 10).await;
    assert_eq!(body["deck_count"], 3);
    assert_eq!(
        names(&body),
        vec!["Jeskai Candidate", "Blue Candidate"],
        "the full set match outranks the partial one: {body}"
    );
    assert_eq!(
        body["suggestions"][0]["reasons"]["colors"],
        json!([
            { "color": "W", "decks": 3 },
            { "color": "R", "decks": 3 },
            { "color": "U", "decks": 2 },
        ]),
        "colors still count decks, most-used first"
    );
    // The profile counts the same decks: white and red in all three, blue in
    // the two Jeskai decks, and Jeskai the identity built twice.
    let profile = &body["profile"];
    assert_eq!(profile["deck_count"], 3);
    assert_eq!(
        profile["colors"],
        json!([
            { "color": "W", "decks": 3 },
            { "color": "R", "decks": 3 },
            { "color": "U", "decks": 2 },
        ])
    );
    assert_eq!(
        profile["top_identity"],
        json!({ "colors": ["W", "U", "R"], "decks": 2 })
    );
    assert_eq!(
        profile["tags"],
        json!([{ "slug": "tokens", "role": null, "decks": 3 }]),
        "no oracle_tags row for the tag, so it falls under no role: {body}"
    );
}

#[sqlx::test]
async fn each_deck_is_matched_on_its_own(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("twodecks").await;
    app.verify_email(&uid).await;

    // A blue deck with four heavily tagged cards and a red-white deck with
    // two lightly tagged ones. Merged, the blue tags would carry most of the
    // profile's weight and bury the red-white match.
    let blue_lead = card("Blue Lead").mono("U").commander_legal();
    let boros_lead = card("Boros Lead").mono("WR").commander_legal();
    let blue_cards: Vec<_> = (0..4)
        .map(|i| card(&format!("Blue {i}")).mono("U"))
        .collect();
    let boros_cards: Vec<_> = (0..2)
        .map(|i| card(&format!("Boros {i}")).mono("WR"))
        .collect();
    let mut tag_rows = Vec::new();
    for c in &blue_cards {
        for tag in ["draw", "counterspell", "mill", "flash", "scry"] {
            tag_rows.push((c.oracle_id().unwrap(), tag));
        }
    }
    for c in &boros_cards {
        for tag in ["equipment", "lifegain"] {
            tag_rows.push((c.oracle_id().unwrap(), tag));
        }
    }
    seed_oracle_tags(&pool, &tag_rows).await;
    let blue_main: Vec<_> = blue_cards
        .iter()
        .map(|c| (c.id(), c.oracle_id().unwrap()))
        .collect();
    let boros_main: Vec<_> = boros_cards
        .iter()
        .map(|c| (c.id(), c.oracle_id().unwrap()))
        .collect();
    let blue_sid = blue_lead.id();
    let boros_sid = boros_lead.id();
    let mut fixtures = vec![blue_lead, boros_lead];
    fixtures.extend(blue_cards);
    fixtures.extend(boros_cards);
    seed_cards(&pool, &fixtures).await;
    build_commander_deck(&app, &token, "Blue", blue_sid, &blue_main).await;
    build_commander_deck(&app, &token, "Boros", boros_sid, &boros_main).await;

    seed_candidate(
        &pool,
        "Blue Candidate",
        "U",
        json!({ "draw": 0.2, "counterspell": 0.2, "mill": 0.2, "flash": 0.2, "scry": 0.2 }),
    )
    .await;
    seed_candidate(
        &pool,
        "Boros Candidate",
        "WR",
        json!({ "equipment": 0.6, "lifegain": 0.4 }),
    )
    .await;

    let body = suggestions(&app, &token, 10).await;
    assert_eq!(
        names(&body),
        vec!["Boros Candidate", "Blue Candidate"],
        "the red-white candidate covers its deck fully, the blue one spreads thin: {body}"
    );
    assert_eq!(
        body["suggestions"][0]["reasons"]["tags"],
        json!([
            { "slug": "equipment", "role": null },
            { "slug": "lifegain", "role": null },
        ]),
        "reasons come from the red-white deck's overlap"
    );
}

#[sqlx::test]
async fn at_most_four_of_one_color_identity_lead_the_list(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("variety").await;
    app.verify_email(&uid).await;

    let lead = card("Blue Lead").mono("U").commander_legal();
    let drawer = card("Drawer").mono("U");
    let other = card("Untracked Burn").mono("R");
    let drawer_pair = (drawer.id(), drawer.oracle_id().unwrap());
    seed_oracle_tags(
        &pool,
        &[
            (drawer_pair.1, "draw"),
            (other.oracle_id().unwrap(), "burn"),
        ],
    )
    .await;
    let lead_sid = lead.id();
    seed_cards(&pool, &[lead, drawer, other]).await;
    build_commander_deck(&app, &token, "Blue", lead_sid, &[drawer_pair]).await;

    // Six mono-blue candidates share the tag and the color; two others share
    // the tag only, so every blue outscores them.
    for i in 0..6 {
        seed_candidate(&pool, &format!("Blue {i}"), "U", json!({ "draw": 1.0 })).await;
    }
    seed_candidate(&pool, "Green Other", "G", json!({ "draw": 1.0 })).await;
    seed_candidate(&pool, "Red Other", "R", json!({ "draw": 1.0 })).await;

    let capped = vec![
        "Blue 0",
        "Blue 1",
        "Blue 2",
        "Blue 3",
        "Green Other",
        "Red Other",
        "Blue 4",
        "Blue 5",
    ];
    assert_eq!(
        names(&suggestions(&app, &token, 10).await),
        capped,
        "four blues lead, the others move up, the capped blues follow"
    );
    assert_eq!(
        names(&suggestions(&app, &token, 100).await),
        capped,
        "a long list keeps every candidate, capped ones after the rest"
    );
    assert_eq!(
        names(&suggestions(&app, &token, 5).await),
        capped[..5],
        "a short list is cut after the cap is applied"
    );
}

#[sqlx::test]
async fn limit_is_honored_and_capped(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (token, uid) = app.register("limiter").await;
    app.verify_email(&uid).await;

    let commander = card("Lead Commander").mono("G").commander_legal();
    let cmd_sid = commander.id();
    let mut fixtures = vec![commander];
    fixtures.extend((0..101).map(|i| card(&format!("Cmdr {i:03}")).mono("G").commander_legal()));
    let candidate_oids: Vec<Uuid> = fixtures[1..]
        .iter()
        .map(|f| f.oracle_id().unwrap())
        .collect();
    seed_cards(&pool, &fixtures).await;
    for oid in &candidate_oids {
        seed_profile(&pool, *oid, json!({ "ramp": 1.0 })).await;
    }
    build_commander_deck(&app, &token, "Green", cmd_sid, &[]).await;

    assert_eq!(names(&suggestions(&app, &token, 1).await).len(), 1);
    assert_eq!(
        names(&suggestions(&app, &token, 500).await).len(),
        100,
        "101 candidates exist; the limit is capped at 100"
    );

    let (status, body) = app.get(COMMANDER_SUGGESTIONS_ROUTE, Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(names(&body).len(), 10, "no limit asks for 10");
}

#[sqlx::test]
async fn every_route_needs_a_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.get(COMMANDER_SUGGESTIONS_ROUTE, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app
        .post(
            COMMANDER_DISMISSALS_ROUTE,
            json!({ "oracle_id": Uuid::nil() }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app
        .delete(&undismiss_commander_route(Uuid::nil()), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = app.delete(COMMANDER_DISMISSALS_ROUTE, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
