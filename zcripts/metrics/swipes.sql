-- Zwipe swipe activity — the shape of swiping: how deep sessions go, how
-- picky people are, whether the gesture misfires, and what a completed deck
-- costs in swipes.
--
-- Built on user_daily_activity (per-user per-day counts) and
-- user_lifetime_counters. Directions: right = primary action (add on search /
-- remove on deck view), left = skip/pass, up = maybeboard, down = undo.
--
--   psql "$DATABASE_URL" -f zcripts/metrics/swipes.sql
--   sudo -u postgres psql zwipe -f zcripts/metrics/swipes.sql
--
-- Read-only. UTC windows; today is partial.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE SWIPE ACTIVITY
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. SESSION DEPTH — swipes per active user-day ──
-- A "session" here is one user's activity on one UTC day. Frames every
-- other number: is a typical sitting 20 swipes or 300?
WITH per_day AS (
    SELECT swipes_right + swipes_left + swipes_up + swipes_down AS swipes
    FROM user_daily_activity
    WHERE swipes_right + swipes_left + swipes_up + swipes_down > 0
)
SELECT
    count(*)                                            AS user_days,
    round(avg(swipes), 1)                               AS avg_swipes,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY swipes) AS median,
    percentile_cont(0.9) WITHIN GROUP (ORDER BY swipes) AS p90,
    max(swipes)                                         AS max
FROM per_day;

\echo
WITH per_day AS (
    SELECT swipes_right + swipes_left + swipes_up + swipes_down AS swipes
    FROM user_daily_activity
    WHERE swipes_right + swipes_left + swipes_up + swipes_down > 0
)
SELECT
    CASE
        WHEN swipes < 10   THEN '1 — 1-9'
        WHEN swipes < 50   THEN '2 — 10-49'
        WHEN swipes < 200  THEN '3 — 50-199'
        WHEN swipes < 500  THEN '4 — 200-499'
        ELSE                    '5 — 500+'
    END                     AS swipes_that_day,
    count(*)                AS user_days
FROM per_day
GROUP BY 1
ORDER BY 1;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. SELECTIVITY TREND — add-rate per week (last 12 weeks) ──
-- add_rate = right / (right + left), the share of decisive swipes that
-- kept the card. A falling rate means people are getting pickier, or the
-- serve is running out of good candidates for them.
SELECT
    date_trunc('week', day)::date                             AS week,
    count(DISTINCT user_id)                                   AS users,
    sum(swipes_right)                                         AS right_,
    sum(swipes_left)                                          AS left_,
    round(100.0 * sum(swipes_right)
          / NULLIF(sum(swipes_right) + sum(swipes_left), 0), 1) AS add_rate_pct
FROM user_daily_activity
WHERE day >= CURRENT_DATE - INTERVAL '12 weeks'
GROUP BY 1
ORDER BY 1;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. UNDO RATE — regret per day (last 14 days) ──
-- down / total. A spike after a release is a UI-regression signal
-- (accidental swipes); the lifetime row is the baseline to compare against.
SELECT
    day,
    sum(swipes_down)                                          AS undos,
    sum(swipes_right + swipes_left + swipes_up + swipes_down) AS swipes,
    round(100.0 * sum(swipes_down)
          / NULLIF(sum(swipes_right + swipes_left
                       + swipes_up + swipes_down), 0), 2)     AS undo_pct
FROM user_daily_activity
WHERE day >= CURRENT_DATE - INTERVAL '14 days'
GROUP BY day
ORDER BY day;

\echo
SELECT
    round(100.0 * sum(swipes_down)
          / NULLIF(sum(swipes_right + swipes_left
                       + swipes_up + swipes_down), 0), 2)     AS lifetime_undo_pct
FROM user_lifetime_counters;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. SELECTIVITY DISTRIBUTION — collectors vs curators ──
-- Per-user lifetime add-rate (right / (right + left)), for users with at
-- least 20 decisive swipes so tiny samples don't smear the buckets.
WITH per_user AS (
    SELECT swipes_right, swipes_left,
           swipes_right::numeric / NULLIF(swipes_right + swipes_left, 0) AS add_rate
    FROM user_lifetime_counters
    WHERE swipes_right + swipes_left >= 20
)
SELECT
    CASE
        WHEN add_rate < 0.10 THEN '1 — <10% (harsh curator)'
        WHEN add_rate < 0.25 THEN '2 — 10-24% (curator)'
        WHEN add_rate < 0.50 THEN '3 — 25-49% (balanced)'
        WHEN add_rate < 0.75 THEN '4 — 50-74% (generous)'
        ELSE                      '5 — 75%+ (collector)'
    END                     AS add_rate_bucket,
    count(*)                AS users,
    sum(swipes_right + swipes_left) AS decisive_swipes
FROM per_user
GROUP BY 1
ORDER BY 1;

\echo
-- How many users are below the 20-swipe threshold (excluded above).
SELECT count(*) AS users_under_20_decisive_swipes
FROM user_lifetime_counters
WHERE swipes_right + swipes_left < 20;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. STREAKS — consecutive active days ──
-- Gaps-and-islands over user_daily_activity. Longest-ever streak
-- distribution, then who's on a live streak right now (habit formed).
WITH days AS (
    SELECT DISTINCT user_id, day FROM user_daily_activity
), grp AS (
    SELECT user_id, day,
           day - (row_number() OVER (PARTITION BY user_id ORDER BY day))::int AS island
    FROM days
), streaks AS (
    SELECT user_id, count(*) AS len, max(day) AS last_day
    FROM grp
    GROUP BY user_id, island
), longest AS (
    SELECT user_id, max(len) AS longest FROM streaks GROUP BY user_id
)
SELECT
    CASE
        WHEN longest = 1  THEN '1 — 1 day'
        WHEN longest <= 3 THEN '2 — 2-3 days'
        WHEN longest <= 6 THEN '3 — 4-6 days'
        ELSE                   '4 — 7+ days'
    END                     AS longest_streak,
    count(*)                AS users
FROM longest
GROUP BY 1
ORDER BY 1;

\echo
-- Live streaks: still counting if active today or yesterday (today may
-- simply not have happened for them yet).
WITH days AS (
    SELECT DISTINCT user_id, day FROM user_daily_activity
), grp AS (
    SELECT user_id, day,
           day - (row_number() OVER (PARTITION BY user_id ORDER BY day))::int AS island
    FROM days
), streaks AS (
    SELECT user_id, count(*) AS len, max(day) AS last_day
    FROM grp
    GROUP BY user_id, island
)
SELECT u.username, s.len AS streak_days, s.last_day
FROM streaks s
JOIN users u ON u.id = s.user_id
WHERE s.last_day >= CURRENT_DATE - 1
ORDER BY s.len DESC, s.last_day DESC
LIMIT 10;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 6. SWIPES PER COMPLETED DECK — effort-to-value ──
-- Among users who completed at least one deck: lifetime swipes divided by
-- decks completed. If this falls across cohorts, suggestion quality is
-- improving (fewer swipes to reach a finished deck).
WITH completers AS (
    SELECT c.user_id,
           c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down AS swipes,
           c.decks_completed,
           (c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down)::numeric
               / c.decks_completed AS swipes_per_deck
    FROM user_lifetime_counters c
    WHERE c.decks_completed > 0
)
SELECT
    count(*)                                                       AS completers,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY swipes_per_deck)   AS median_swipes_per_deck,
    round(avg(swipes_per_deck), 1)                                 AS avg_swipes_per_deck,
    min(swipes_per_deck)::int                                      AS min,
    max(swipes_per_deck)::int                                      AS max
FROM completers;

\echo
WITH completers AS (
    SELECT c.user_id,
           c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down AS swipes,
           c.decks_completed
    FROM user_lifetime_counters c
    WHERE c.decks_completed > 0
)
SELECT u.username, cm.swipes, cm.decks_completed,
       round(cm.swipes::numeric / cm.decks_completed, 0) AS swipes_per_deck
FROM completers cm
JOIN users u ON u.id = cm.user_id
ORDER BY swipes_per_deck DESC
LIMIT 15;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF SWIPE ACTIVITY REPORT
\echo ════════════════════════════════════════════════════════════════
