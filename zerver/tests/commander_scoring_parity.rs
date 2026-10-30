//! The commander suggestions scoring statement ranks, scores and explains
//! candidates exactly as the statement before it did. That statement looked up
//! each shared tag's role in SQL through the role membership CTEs, with the
//! recursive step joined on `= ANY(parent_ids)`, and probed each deck profile
//! with jsonb `?` per candidate. The current one hash-joins exploded deck tags
//! and returns bare slugs, and the service attaches roles from `tag_roles`.
//! Both run here over one seeded catalog, deck set and candidate pool: tags
//! under one role, two roles and none, a nested subtree, twin candidates that
//! tie, a colorless candidate, and the token, illegal, excluded and dismissed
//! ones the pool leaves out.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "a test: helpers outside #[test] fns unwrap and index, and the seeded generator does small modular math; a panic is the failure report"
)]

mod common;

use std::collections::BTreeSet;

use common::{TestApp, card, seed_cards};
use serde_json::{Map, Value, json};
use sqlx::Row;
use uuid::Uuid;
use zwipe::domain::card::ports::CardRepository;
use zwipe::domain::deck::models::deck::commander_suggestions::{SharedTag, shared_tags};
use zwipe::outbound::sqlx::card::helpers::commander_scoring::SCORE_COMMANDER_SUGGESTIONS_SQL;
use zwipe::outbound::sqlx::card::helpers::oracle_tag_groups::RoleMembershipBinds;
use zwipe::outbound::sqlx::postgres::Postgres;

/// The scoring statement as it stood before the rewrite, membership CTEs
/// inlined. Binds: the six membership arrays, then the decks, exclusions,
/// user, dismissal days, tag limit, color weight and row limit.
const REFERENCE_SQL: &str = r"WITH RECURSIVE roots(role, root_slug) AS (
             SELECT r, s FROM unnest($1::text[], $2::text[]) AS t(r, s)
         ),
         excl(role, slug) AS (
             SELECT r, s FROM unnest($5::text[], $6::text[]) AS e(r, s)
         ),
         subtree(role, id, slug) AS (
             SELECT r.role, ot.id, ot.slug FROM roots r JOIN oracle_tags ot ON ot.slug = r.root_slug
             UNION
             SELECT st.role, c.id, c.slug FROM subtree st JOIN oracle_tags c ON st.id = ANY(c.parent_ids)
         ),
         membership(role, slug) AS (
             SELECT role, slug FROM (
                 SELECT DISTINCT role, slug FROM subtree
                 UNION
                 -- exact leaf overrides: matched as-is, no subtree expansion
                 SELECT r, s FROM unnest($3::text[], $4::text[]) AS o(r, s)
             ) m
             -- subtract exact (role, slug) exclusions dragged in by the hierarchy
             WHERE NOT EXISTS (SELECT 1 FROM excl e WHERE e.role = m.role AND e.slug = m.slug)
         ),
             decks AS (
                 SELECT d.ord,
                        ARRAY(SELECT jsonb_array_elements_text(d.value->'colors')) AS colors,
                        COALESCE(d.value->'profile', '{}'::jsonb) AS profile
                 FROM jsonb_array_elements($7::jsonb) WITH ORDINALITY d(value, ord)
             ),
             candidates AS (
                 SELECT p.oracle_id, p.profile, lc.name,
                        COALESCE(lc.color_identity, '{}') AS color_identity
                 FROM commander_tag_profiles p
                 JOIN latest_cards lc ON lc.oracle_id = p.oracle_id
                 WHERE lc.layout <> 'token'
                   AND lc.legalities->>'commander' = 'legal'
                   AND NOT (p.oracle_id = ANY($8::uuid[]))
                   AND NOT EXISTS (
                       SELECT 1 FROM user_commander_dismissals d
                       WHERE d.user_id = $9 AND d.oracle_id = p.oracle_id
                         AND d.dismissed_at > now() - make_interval(days => $10)
                   )
             ),
             overlap AS (
                 SELECT c.oracle_id, d.ord AS deck, t.key AS oracle_tag,
                        (d.profile->>t.key)::float8 * t.value::float8 AS w
                 FROM candidates c
                 CROSS JOIN decks d
                 CROSS JOIN LATERAL jsonb_each_text(c.profile) t
                 WHERE d.profile ? t.key
             ),
             deck_tag_score AS (
                 SELECT oracle_id, deck, sum(w) AS s FROM overlap GROUP BY oracle_id, deck
             ),
             best_deck AS (
                 SELECT DISTINCT ON (oracle_id) oracle_id, deck, s
                 FROM deck_tag_score
                 ORDER BY oracle_id, s DESC, deck
             ),
             color_score AS (
                 SELECT c.oracle_id,
                        max(CASE
                            WHEN cardinality(c.color_identity) = 0 AND cardinality(d.colors) = 0
                                THEN 1.0::float8
                            WHEN cardinality(c.color_identity) = 0 OR cardinality(d.colors) = 0
                                THEN 0.0::float8
                            ELSE (SELECT count(*) FROM unnest(c.color_identity) x
                                  WHERE x = ANY(d.colors))::float8
                               / (SELECT count(DISTINCT x)
                                  FROM unnest(c.color_identity || d.colors) x)::float8
                        END) AS s
                 FROM candidates c
                 CROSS JOIN decks d
                 GROUP BY c.oracle_id
             ),
             tag_role AS (
                 SELECT slug, min(role) AS role FROM membership GROUP BY slug
             ),
             shared_tags AS (
                 SELECT o.oracle_id,
                        jsonb_agg(
                            jsonb_build_object('slug', o.oracle_tag, 'role', tr.role)
                            ORDER BY o.w DESC, o.oracle_tag
                        ) AS tags
                 FROM (
                     SELECT o.oracle_id, o.oracle_tag, o.w,
                            row_number() OVER (
                                PARTITION BY o.oracle_id ORDER BY o.w DESC, o.oracle_tag
                            ) AS rn
                     FROM overlap o
                     JOIN best_deck bd ON bd.oracle_id = o.oracle_id AND bd.deck = o.deck
                 ) o
                 LEFT JOIN tag_role tr ON tr.slug = o.oracle_tag
                 WHERE o.rn <= $11
                 GROUP BY o.oracle_id
             )
             SELECT c.oracle_id,
                    c.color_identity,
                    COALESCE(bd.s, 0) + $12 * COALESCE(cs.s, 0) AS score,
                    COALESCE(st.tags, '[]'::jsonb) AS tags
             FROM candidates c
             LEFT JOIN best_deck bd ON bd.oracle_id = c.oracle_id
             LEFT JOIN color_score cs ON cs.oracle_id = c.oracle_id
             LEFT JOIN shared_tags st ON st.oracle_id = c.oracle_id
             ORDER BY score DESC, c.name ASC, c.oracle_id
             LIMIT $13";

const TAG_LIMIT: i64 = 5;
const COLOR_WEIGHT: f64 = 0.15;
const DISMISSAL_DAYS: i32 = 7;
const ROW_LIMIT: i64 = 100;

/// Every slug the fixture decks and candidates draw from. `typal-goblin` is in
/// no catalog row, so it falls under no role.
const SLUGS: [&str; 13] = [
    "removal",
    "spot-removal",
    "lockdown",
    "ramp",
    "mana-producer",
    "ramp-removal",
    "burn",
    "lifegain",
    "gives-haste",
    "sacrifice-outlet",
    "repeatable-token-generator",
    "pure-draw",
    "typal-goblin",
];

/// A small deterministic generator, so the fixture is varied but fixed.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    /// A normalized profile over some of `SLUGS`, plus `extra` slugs.
    fn profile(&mut self, extra: &[&str]) -> Value {
        let mut raw: Vec<(String, f64)> = Vec::new();
        for slug in SLUGS {
            if self.next().is_multiple_of(3) {
                let weight = u32::try_from(self.next() % 97 + 3).unwrap();
                raw.push((slug.to_string(), f64::from(weight)));
            }
        }
        raw.extend(extra.iter().map(|s| (s.to_string(), 7.0)));
        let total: f64 = raw.iter().map(|(_, w)| w).sum();
        let map: Map<String, Value> = raw
            .into_iter()
            .map(|(slug, w)| (slug, json!(w / total)))
            .collect();
        Value::Object(map)
    }
}

/// Writes one catalog row with the given parent ids.
async fn seed_catalog_tag(pool: &sqlx::PgPool, id: u128, slug: &str, parents: &[u128]) {
    let parents: Vec<Uuid> = parents.iter().copied().map(Uuid::from_u128).collect();
    sqlx::query("INSERT INTO oracle_tags (id, slug, label, parent_ids) VALUES ($1, $2, $2, $3)")
        .bind(Uuid::from_u128(id))
        .bind(slug)
        .bind(&parents)
        .execute(pool)
        .await
        .unwrap();
}

/// Writes one commander's precomputed tag profile.
async fn seed_profile(pool: &sqlx::PgPool, oracle_id: Uuid, profile: &Value) {
    sqlx::query("INSERT INTO commander_tag_profiles (oracle_id, profile) VALUES ($1, $2)")
        .bind(oracle_id)
        .bind(sqlx::types::Json(profile))
        .execute(pool)
        .await
        .unwrap();
}

/// One scored row: oracle id, color identity, score, and the shared tags.
type Scored = (Uuid, Vec<String>, f64, Vec<SharedTag>);

#[sqlx::test]
async fn scoring_matches_the_reference_statement(pool: sqlx::PgPool) {
    let app = TestApp::new(pool.clone());
    let (_token, user_id) = app.register("scorer").await;
    let user_id: Uuid = user_id.parse().unwrap();

    // removal and lockdown are removal roots, lockdown a stax root too;
    // ramp-removal sits under both ramp and spot-removal.
    let catalog: [(u128, &str, &[u128]); 12] = [
        (0x01, "removal", &[]),
        (0x02, "spot-removal", &[0x01]),
        (0x03, "lockdown", &[]),
        (0x04, "ramp", &[]),
        (0x05, "mana-producer", &[]),
        (0x06, "ramp-removal", &[0x04, 0x02]),
        (0x07, "burn", &[]),
        (0x08, "lifegain", &[]),
        (0x09, "gives-haste", &[]),
        (0x0a, "sacrifice-outlet", &[]),
        (0x0b, "repeatable-token-generator", &[]),
        (0x0c, "pure-draw", &[]),
    ];
    for (id, slug, parents) in catalog {
        seed_catalog_tag(&pool, id, slug, parents).await;
    }

    let mut rng = Lcg(42);
    let deck_colors: [&[&str]; 6] = [
        &["W", "U"],
        &["B", "R", "G"],
        &[],
        &["G"],
        &["U", "R"],
        &["W"],
    ];
    let mut decks: Vec<Value> = deck_colors
        .iter()
        .map(|colors| json!({ "colors": colors, "profile": rng.profile(&[]) }))
        .collect();
    // A mono-green deck the twins below match on purpose, with a tag under
    // two roles at each depth and one under none.
    decks[3]["profile"] = json!({ "ramp-removal": 0.4, "lockdown": 0.35, "typal-goblin": 0.25 });
    // A deck whose cards carried no usable tags.
    decks[5]["profile"] = Value::Null;
    let decks = Value::Array(decks);

    let identities = [
        "", "W", "U", "B", "R", "G", "WU", "BR", "WUBRG", "GW", "URG",
    ];
    let mut cards = Vec::new();
    let mut profiles = Vec::new();
    for i in 0..40 {
        let fixture = card(&format!("Candidate {i:02}"))
            .mono(identities[i % identities.len()])
            .commander_legal();
        // Some candidates carry a tag no deck has.
        let extra: &[&str] = if i % 4 == 0 { &["cycle-off-deck"] } else { &[] };
        profiles.push((fixture.oracle_id().unwrap(), rng.profile(extra)));
        cards.push(fixture);
    }
    // Twins: the same identity and profile, so they tie and fall back to name.
    let twin_profile = json!({ "ramp-removal": 0.5, "lockdown": 0.3, "typal-goblin": 0.2 });
    let mut twin_ids = Vec::new();
    for name in ["Twin B", "Twin A"] {
        let fixture = card(name).mono("G").commander_legal();
        twin_ids.push(fixture.oracle_id().unwrap());
        profiles.push((fixture.oracle_id().unwrap(), twin_profile.clone()));
        cards.push(fixture);
    }
    // The pool leaves these out.
    let token = card("Token Lead").mono("G").commander_legal().token();
    let illegal = card("Banned Lead").mono("G");
    let excluded = card("Excluded Lead").mono("G").commander_legal();
    let dismissed = card("Dismissed Lead").mono("G").commander_legal();
    let left_out = [&token, &illegal, &excluded, &dismissed].map(|c| c.oracle_id().unwrap());
    for fixture in [token, illegal, excluded, dismissed] {
        profiles.push((fixture.oracle_id().unwrap(), twin_profile.clone()));
        cards.push(fixture);
    }
    seed_cards(&pool, &cards).await;
    for (oracle_id, profile) in &profiles {
        seed_profile(&pool, *oracle_id, profile).await;
    }
    sqlx::query("INSERT INTO user_commander_dismissals (user_id, oracle_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(left_out[3])
        .execute(&pool)
        .await
        .unwrap();
    let exclude = vec![left_out[2]];

    let reference = run_reference(&pool, &decks, &exclude, user_id).await;
    let current = run_current(&pool, &decks, &exclude, user_id).await;

    assert_eq!(reference.len(), 42, "every legal, kept candidate scores");
    assert_eq!(current.len(), reference.len(), "row count");
    for (rank, (want, got)) in reference.iter().zip(&current).enumerate() {
        assert_eq!(got.0, want.0, "rank {rank}: oracle id");
        assert_eq!(got.1, want.1, "rank {rank}: color identity");
        assert!(
            (got.2 - want.2).abs() < 1e-12,
            "rank {rank}: score {} vs {}",
            got.2,
            want.2
        );
        assert_eq!(got.3, want.3, "rank {rank}: shared tags");
    }

    // The fixture reaches the cases it is built around.
    let ids: BTreeSet<Uuid> = reference.iter().map(|r| r.0).collect();
    assert!(left_out.iter().all(|id| !ids.contains(id)), "left out");
    let tags: Vec<&SharedTag> = reference.iter().flat_map(|r| &r.3).collect();
    let role_of = |slug: &str| tags.iter().find(|t| t.slug == slug).map(|t| t.role.clone());
    assert_eq!(
        role_of("lockdown"),
        Some(Some("removal".to_string())),
        "first of two roles"
    );
    assert_eq!(
        role_of("ramp-removal"),
        Some(Some("ramp".to_string())),
        "nested, under two roles"
    );
    assert_eq!(role_of("typal-goblin"), Some(None), "under no role");
    assert!(
        reference.iter().any(|r| r.3.is_empty()),
        "a candidate shares nothing"
    );
    let twins: Vec<(Uuid, f64)> = reference
        .iter()
        .filter(|r| twin_ids.contains(&r.0))
        .map(|r| (r.0, r.2))
        .collect();
    assert_eq!(twins.len(), 2, "both twins score");
    assert_eq!(twins[0].1, twins[1].1, "twins tie");
    assert_eq!(twins[0].0, twin_ids[1], "a tie falls back to name");

    // The repository method returns the statement's ranking and slugs.
    let repo = Postgres { pool: pool.clone() };
    let scored = repo
        .score_commander_suggestions(user_id, &decks, &exclude, ROW_LIMIT)
        .await
        .unwrap();
    let repo_rows: Vec<(Uuid, Vec<String>)> = scored
        .into_iter()
        .map(|s| (s.oracle_id, s.tag_slugs))
        .collect();
    let current_rows: Vec<(Uuid, Vec<String>)> = current
        .iter()
        .map(|r| (r.0, r.3.iter().map(|t| t.slug.clone()).collect()))
        .collect();
    assert_eq!(repo_rows, current_rows, "repository vs statement");
}

/// Runs the reference statement; its tags come back as `{slug, role}` objects.
async fn run_reference(
    pool: &sqlx::PgPool,
    decks: &Value,
    exclude: &[Uuid],
    user_id: Uuid,
) -> Vec<Scored> {
    let binds = RoleMembershipBinds::new();
    binds
        .bind(sqlx::query(REFERENCE_SQL))
        .bind(decks)
        .bind(exclude)
        .bind(user_id)
        .bind(DISMISSAL_DAYS)
        .bind(TAG_LIMIT)
        .bind(COLOR_WEIGHT)
        .bind(ROW_LIMIT)
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| {
            let tags: Vec<Value> = serde_json::from_value(row.get(3)).unwrap();
            let tags = tags
                .into_iter()
                .map(|t| SharedTag {
                    slug: t["slug"].as_str().unwrap().to_string(),
                    role: t["role"].as_str().map(str::to_string),
                })
                .collect();
            (row.get(0), row.get(1), row.get(2), tags)
        })
        .collect()
}

/// Runs the current statement and attaches roles the way the service does:
/// from `tag_roles` over every key of the deck profiles.
async fn run_current(
    pool: &sqlx::PgPool,
    decks: &Value,
    exclude: &[Uuid],
    user_id: Uuid,
) -> Vec<Scored> {
    let slugs: Vec<String> = decks
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|deck| deck["profile"].as_object())
        .flat_map(|profile| profile.keys().cloned())
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect();
    let repo = Postgres { pool: pool.clone() };
    let tag_roles = repo.tag_roles(&slugs).await.unwrap();
    sqlx::query(SCORE_COMMANDER_SUGGESTIONS_SQL)
        .bind(decks)
        .bind(exclude)
        .bind(user_id)
        .bind(DISMISSAL_DAYS)
        .bind(TAG_LIMIT)
        .bind(COLOR_WEIGHT)
        .bind(ROW_LIMIT)
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| {
            let slugs: Vec<String> = row.get(3);
            (
                row.get(0),
                row.get(1),
                row.get(2),
                shared_tags(slugs, &tag_roles),
            )
        })
        .collect()
}
