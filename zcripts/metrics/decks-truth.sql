-- Build-independent deck truth — read this until 1.0.3 telemetry reaches users.
--
-- Run it:
--   sudo -u postgres psql zwipe -f zcripts/metrics/decks-truth.sql
--   psql "$DATABASE_URL" -f zcripts/metrics/decks-truth.sql
--
-- Everything here comes from the `decks` / `deck_cards` tables, which are
-- written server-side on every create regardless of the user's app build. So
-- unlike the swipe/search counters, these numbers reflect ALL users right now.
-- Read-only.

\pset border 2
\pset null '·'
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  DECK TRUTH — server-side, build-independent
\echo ════════════════════════════════════════════════════════════════


\echo
\echo ── 1. DECKS BUILT PER DAY + distinct builders ──
-- The real activation pulse: how many decks, by how many distinct people.
SELECT
    created_at::date            AS day,
    count(*)                    AS decks,
    count(DISTINCT user_id)     AS builders
FROM decks
GROUP BY 1
ORDER BY 1;


\echo
\echo ── 2. ACTIVATION — of all signups, how many ever built a deck ──
SELECT
    (SELECT count(*) FROM users)                       AS signups,
    (SELECT count(DISTINCT user_id) FROM decks)        AS builders,
    round(100.0 * (SELECT count(DISTINCT user_id) FROM decks)
          / NULLIF((SELECT count(*) FROM users), 0), 1) AS builder_pct;


\echo
\echo ── 3. DECKS PER BUILDER (excludes 0-deck users) ──
-- Among people who built anything, do they build one and stop or keep going?
WITH per_user AS (
    SELECT user_id, count(*) AS decks FROM decks GROUP BY user_id
)
SELECT
    CASE WHEN decks = 1 THEN '1 deck'
         WHEN decks <= 4 THEN '2-4 decks'
         ELSE '5+ decks' END    AS bucket,
    count(*)                    AS builders
FROM per_user
GROUP BY 1
ORDER BY 1;


\echo
\echo ── 4. DECK SIZE — main board, per deck ──
-- Are these real decks or stubs? Size distribution of the 'deck' board.
WITH sizes AS (
    SELECT d.id,
           COALESCE(sum(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) AS cards
    FROM decks d
    LEFT JOIN deck_cards dc ON dc.deck_id = d.id
    GROUP BY d.id
)
SELECT
    count(*)                                            AS decks,
    round(avg(cards), 1)                                AS avg_cards,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY cards)  AS median_cards,
    count(*) FILTER (WHERE cards >= 100)                AS at_100_plus,
    count(*) FILTER (WHERE cards BETWEEN 1 AND 9)       AS barely_started,
    max(cards)                                          AS max_cards
FROM sizes;


\echo
\echo ── 5. FORMAT split ──
SELECT COALESCE(format, '· (unset)') AS format, count(*) AS decks
FROM decks GROUP BY format ORDER BY decks DESC;


\echo
\echo ── 6. TOP COMMANDERS (top 15) ──
SELECT sd.name AS commander, count(*) AS decks
FROM decks d
JOIN scryfall_data sd ON sd.id = d.commander_id
WHERE d.commander_id IS NOT NULL
GROUP BY sd.name
ORDER BY decks DESC
LIMIT 15;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END — these numbers are real regardless of app build
\echo ════════════════════════════════════════════════════════════════
