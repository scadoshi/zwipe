//! The commander suggestions scoring statement, kept apart from the repository
//! method so a parity test can run it beside a reference query.

/// Ranks the commanders in `commander_tag_profiles` against a user's decks.
///
/// Score is the best deck's sum over shared tags of deck weight × commander
/// weight, plus `$6` × the best deck's Jaccard match of color identities
/// (|∩| / |∪|; colorless against colorless is 1, colorless against colored 0).
/// The two bests may come from different decks. `tags` holds the slugs of the
/// top `$5` tags shared with the best-tag deck by that product, strongest
/// first; each is a key of that deck's profile, so its card role is looked up
/// with the profiles' other keys rather than here.
///
/// The deck tags and the candidate tags each explode into one row per tag
/// once and meet in a single hash join. `shared_tags` is materialized: inlined,
/// the planner reruns it per candidate.
///
/// Binds, in order:
/// 1. `jsonb`: the decks, `[{"colors": [WUBRG], "profile": {otag: weight} | null}]`
/// 2. `uuid[]`: oracle ids to leave out
/// 3. `uuid`: the user, whose live dismissals are left out
/// 4. `int4`: how many days a dismissal lasts
/// 5. `int8`: shared tags kept per candidate
/// 6. `float8`: the color term's weight
/// 7. `int8`: the row limit
///
/// Columns: `oracle_id uuid`, `color_identity text[]`, `score float8`,
/// `tags text[]`.
pub const SCORE_COMMANDER_SUGGESTIONS_SQL: &str = "WITH
     decks AS (
         SELECT d.ord,
                ARRAY(SELECT jsonb_array_elements_text(d.value->'colors')) AS colors,
                COALESCE(d.value->'profile', '{}'::jsonb) AS profile
         FROM jsonb_array_elements($1::jsonb) WITH ORDINALITY d(value, ord)
     ),
     deck_tags AS MATERIALIZED (
         SELECT d.ord, k.key, k.value::float8 AS w
         FROM decks d, jsonb_each_text(d.profile) k
         -- an unprofiled deck sends a JSON null, which jsonb_each_text rejects
         WHERE jsonb_typeof(d.profile) = 'object'
     ),
     candidates AS (
         SELECT p.oracle_id, p.profile, lc.name,
                COALESCE(lc.color_identity, '{}') AS color_identity
         FROM commander_tag_profiles p
         JOIN latest_cards lc ON lc.oracle_id = p.oracle_id
         WHERE lc.layout <> 'token'
           AND lc.legalities->>'commander' = 'legal'
           AND NOT (p.oracle_id = ANY($2::uuid[]))
           AND NOT EXISTS (
               SELECT 1 FROM user_commander_dismissals d
               WHERE d.user_id = $3 AND d.oracle_id = p.oracle_id
                 AND d.dismissed_at > now() - make_interval(days => $4)
           )
     ),
     candidate_tags AS MATERIALIZED (
         SELECT c.oracle_id, t.key, t.value::float8 AS w
         FROM candidates c, jsonb_each_text(c.profile) t
     ),
     overlap AS (
         SELECT ct.oracle_id, dt.ord AS deck, ct.key AS oracle_tag, dt.w * ct.w AS w
         FROM candidate_tags ct
         JOIN deck_tags dt ON dt.key = ct.key
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
     shared_tags AS MATERIALIZED (
         SELECT o.oracle_id,
                array_agg(o.oracle_tag ORDER BY o.w DESC, o.oracle_tag) AS tags
         FROM (
             SELECT o.oracle_id, o.oracle_tag, o.w,
                    row_number() OVER (
                        PARTITION BY o.oracle_id ORDER BY o.w DESC, o.oracle_tag
                    ) AS rn
             FROM overlap o
             JOIN best_deck bd ON bd.oracle_id = o.oracle_id AND bd.deck = o.deck
         ) o
         WHERE o.rn <= $5
         GROUP BY o.oracle_id
     )
     SELECT c.oracle_id,
            c.color_identity,
            COALESCE(bd.s, 0) + $6 * COALESCE(cs.s, 0) AS score,
            COALESCE(st.tags, '{}'::text[]) AS tags
     FROM candidates c
     LEFT JOIN best_deck bd ON bd.oracle_id = c.oracle_id
     LEFT JOIN color_score cs ON cs.oracle_id = c.oracle_id
     LEFT JOIN shared_tags st ON st.oracle_id = c.oracle_id
     ORDER BY score DESC, c.name ASC, c.oracle_id
     LIMIT $7";
