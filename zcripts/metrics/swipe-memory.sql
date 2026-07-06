-- Zwipe swipe memory — are durable skips (deck_card_suppressions) being used,
-- and is any deck near the 5,000-row cap?
--
-- Suppressions are the 1.3.0 per-swipe durable-skip feature: cards the
-- deck-aware search must not serve a deck again. source = 'skip' (Add-screen
-- left swipe) or 'removal' (deliberate single-card removal).
--
--   psql "$DATABASE_URL" -f zcripts/metrics/swipe-memory.sql
--   sudo -u postgres psql zwipe -f zcripts/metrics/swipe-memory.sql
--
-- Read-only. UTC. Empty until users are on a 1.3.0+ build.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE SWIPE MEMORY
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. TOTALS — suppression volume + source split ──
SELECT
    count(*)                                       AS suppressions,
    count(DISTINCT deck_id)                        AS decks_using,
    count(*) FILTER (WHERE source = 'skip')        AS from_skip,
    count(*) FILTER (WHERE source = 'removal')     AS from_removal,
    min(suppressed_at)::date                       AS first,
    max(suppressed_at)::date                       AS latest
FROM deck_card_suppressions;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. ADOPTION — share of active decks using durable skips ──
-- Denominator = decks touched in the last 30 days (updated_at moves on edits).
WITH active AS (
    SELECT id FROM decks WHERE updated_at >= now() - INTERVAL '30 days'
)
SELECT
    (SELECT count(*) FROM active)                                    AS active_decks_30d,
    (SELECT count(DISTINCT s.deck_id) FROM deck_card_suppressions s
      JOIN active a ON a.id = s.deck_id)                             AS of_which_suppressing,
    round(100.0 * (SELECT count(DISTINCT s.deck_id)
                   FROM deck_card_suppressions s
                   JOIN active a ON a.id = s.deck_id)
          / NULLIF((SELECT count(*) FROM active), 0), 1)             AS pct;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. PER-DECK DISTRIBUTION — how deep do skip lists get? ──
-- The ingest cap is 5,000 per deck (oldest evicted). Anyone in the top
-- bucket is churning through a huge share of the pool for their colors.
WITH per_deck AS (
    SELECT deck_id, count(*) AS n
    FROM deck_card_suppressions
    GROUP BY deck_id
)
SELECT
    CASE
        WHEN n < 10    THEN '1 — 1-9'
        WHEN n < 50    THEN '2 — 10-49'
        WHEN n < 200   THEN '3 — 50-199'
        WHEN n < 1000  THEN '4 — 200-999'
        WHEN n < 4000  THEN '5 — 1000-3999'
        ELSE                '6 — 4000+ (near cap)'
    END                     AS suppressions_bucket,
    count(*)                AS decks
FROM per_deck
GROUP BY 1
ORDER BY 1;

\echo
-- Largest skip lists, with owner and deck for recognizability.
SELECT
    u.username,
    d.name                     AS deck,
    count(*)                   AS suppressions,
    count(*) FILTER (WHERE s.source = 'removal') AS from_removal,
    max(s.suppressed_at)::date AS latest
FROM deck_card_suppressions s
JOIN decks d ON d.id = s.deck_id
JOIN users u ON u.id = d.user_id
GROUP BY u.username, d.name
ORDER BY suppressions DESC
LIMIT 10;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. DAILY VOLUME — suppressions written per day (last 14 days) ──
-- Tracks uptake of the feature after each store build rolls out.
SELECT
    suppressed_at::date         AS day,
    count(*)                    AS suppressions,
    count(DISTINCT deck_id)     AS decks
FROM deck_card_suppressions
WHERE suppressed_at >= CURRENT_DATE - INTERVAL '14 days'
GROUP BY 1
ORDER BY 1;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF SWIPE MEMORY REPORT
\echo ════════════════════════════════════════════════════════════════
