-- Repeat-builder retention — server-side, build-independent.
--
-- Run it:
--   sudo -u postgres psql zwipe -f zcripts/metrics/retention.sql
--   psql "$DATABASE_URL" -f zcripts/metrics/retention.sql
--
-- "Retention" here = did a builder come BACK on a later day to build again. It's
-- derived purely from decks.created_at, so it's real for every user regardless
-- of app build. Re-run weekly and watch the repeat-builder count climb (or not).
-- Read-only.
--
-- Note: a user building 3 decks in one sitting is NOT retention — we count
-- distinct calendar days (UTC) a user created decks on. >1 day = they returned.

\pset border 2
\pset null '·'
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  RETENTION — repeat deck-builders
\echo ════════════════════════════════════════════════════════════════


\echo
\echo ── 1. HEADLINE — how many builders came back on a later day ──
-- one_day_only = built everything in a single session/day (not yet retained).
-- repeat_builders = created decks on 2+ distinct days = genuine return.
WITH per_user AS (
    SELECT user_id,
           count(*)                          AS decks,
           count(DISTINCT created_at::date)  AS build_days
    FROM decks
    GROUP BY user_id
)
SELECT
    count(*)                                       AS builders,
    count(*) FILTER (WHERE build_days = 1)         AS one_day_only,
    count(*) FILTER (WHERE build_days >= 2)        AS repeat_builders,
    round(100.0 * count(*) FILTER (WHERE build_days >= 2)
          / NULLIF(count(*), 0), 1)                AS repeat_pct
FROM per_user;


\echo
\echo ── 2. PER-BUILDER timeline (most recently active first) ──
-- first/last build day, how many distinct days they built, and the span. A
-- nonzero span_days = they returned at least once after their first deck.
SELECT
    u.username,
    count(*)                            AS decks,
    count(DISTINCT d.created_at::date)  AS build_days,
    min(d.created_at)::date             AS first_build,
    max(d.created_at)::date             AS last_build,
    (max(d.created_at)::date - min(d.created_at)::date) AS span_days
FROM decks d
JOIN users u ON u.id = d.user_id
GROUP BY u.username
ORDER BY last_build DESC, decks DESC;


\echo
\echo ── 3. RETURN GAP — for repeat builders, days from 1st to latest deck ──
-- Buckets the span between a repeat builder's first and most recent deck.
WITH per_user AS (
    SELECT user_id,
           (max(created_at)::date - min(created_at)::date) AS span_days,
           count(DISTINCT created_at::date)                AS build_days
    FROM decks
    GROUP BY user_id
)
SELECT
    CASE
        WHEN span_days = 0     THEN 'same day only'
        WHEN span_days = 1     THEN 'next day'
        WHEN span_days <= 7    THEN 'within a week'
        WHEN span_days <= 30   THEN 'within a month'
        ELSE                        'month+'
    END                         AS return_gap,
    count(*)                    AS builders
FROM per_user
GROUP BY 1
ORDER BY min(span_days);

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END — re-run weekly; watch repeat_builders / repeat_pct move
\echo ════════════════════════════════════════════════════════════════
