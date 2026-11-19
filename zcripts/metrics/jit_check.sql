-- JIT check: does Postgres spend more time compiling zerver's per-user queries than running them?
--
-- Prints the server's JIT settings, then explains fetch_profiled_decks (the deck
-- read behind suggested commanders) for the user with the most decks, twice: as the server is
-- configured, and with JIT off for this transaction only. If the first plan's JIT section shows
-- hundreds of milliseconds while the second runs in a few, zerver pays the compile cost.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/jit_check.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/jit_check.sql > /tmp/jit-check.txt
--
-- Read-only: one READ ONLY transaction with a statement timeout, SHOW, SELECT and EXPLAIN only.
-- The user is picked inside psql and never printed: the query runs as a generic prepared
-- statement, so its plan shows $1 rather than the id. Keep the statement in step with
-- fetch_profiled_decks in zerver/src/lib/outbound/sqlx/deck/mod.rs.

\set ON_ERROR_STOP on
\set QUIET on
\pset border 2
\timing off

BEGIN TRANSACTION READ ONLY;
SET LOCAL statement_timeout = '60s';
SET LOCAL plan_cache_mode = force_generic_plan;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE JIT CHECK
\echo ════════════════════════════════════════════════════════════════
SELECT version() AS postgres;

\echo
\echo ── 1. JIT settings ──
SELECT name, setting, source
FROM pg_settings
WHERE name IN ('jit', 'jit_above_cost', 'jit_optimize_above_cost', 'jit_inline_above_cost',
               'jit_provider')
ORDER BY name;
SELECT pg_jit_available() AS jit_available;

\echo
\echo ── 2. The user with the most decks: their deck count only ──
SELECT user_id AS top_user FROM decks GROUP BY user_id ORDER BY count(*) DESC, user_id LIMIT 1 \gset
SELECT count(*) AS decks FROM decks WHERE user_id = :'top_user';

PREPARE profiled_decks(uuid) AS
SELECT s.oracle_id AS commander_oracle_id,
       s.color_identity AS commander_colors,
       (SELECT array_agg(DISTINCT ci)
          FROM scryfall_data sci, unnest(sci.color_identity) AS ci
         WHERE sci.id IN (SELECT dc2.scryfall_data_id FROM deck_cards dc2
                           WHERE dc2.deck_id = d.id AND dc2.board = 'deck'))
           AS card_colors,
       (SELECT ARRAY[
                   COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'W' = ANY(m.color_identity)),
                   COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'U' = ANY(m.color_identity)),
                   COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'B' = ANY(m.color_identity)),
                   COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'R' = ANY(m.color_identity)),
                   COUNT(DISTINCT m.oracle_id) FILTER (WHERE 'G' = ANY(m.color_identity)),
                   COUNT(DISTINCT m.oracle_id) FILTER (WHERE cardinality(m.color_identity) = 0)]
          FROM (SELECT cc.oracle_id, cs.color_identity
                  FROM deck_cards cc
                  JOIN scryfall_data cs ON cs.id = cc.scryfall_data_id
                 WHERE cc.deck_id = d.id AND cc.board = 'deck'
                UNION ALL
                SELECT cz.oracle_id, cz.color_identity
                  FROM scryfall_data cz
                 WHERE cz.id IN (d.commander_id, d.partner_commander_id, d.background_id)
                   AND cz.oracle_id IS NOT NULL) AS m)
           AS card_color_counts,
       COALESCE(array_agg(dc.oracle_id) FILTER (WHERE dc.board = 'deck'), '{}')
           AS mainboard_oracle_ids
FROM decks d
LEFT JOIN scryfall_data s ON s.id = d.commander_id AND s.oracle_id IS NOT NULL
LEFT JOIN deck_cards dc ON dc.deck_id = d.id
WHERE d.user_id = $1
GROUP BY d.id, s.oracle_id, s.color_identity
HAVING s.oracle_id IS NOT NULL OR bool_or(dc.board = 'deck')
ORDER BY d.updated_at DESC, d.id;

\echo
\echo ── 3. fetch_profiled_decks as configured: see the JIT section at the end ──
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) EXECUTE profiled_decks(:'top_user');

\echo
\echo ── 4. The same with JIT off for this transaction ──
SET LOCAL jit = off;
EXPLAIN (ANALYZE, BUFFERS, SUMMARY) EXECUTE profiled_decks(:'top_user');

DEALLOCATE profiled_decks;
COMMIT;
