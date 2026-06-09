-- Zwipe usage overview — single-run report against the production database.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/overview.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/overview.sql
--
-- Every section prints a labeled header (via \echo) so the output is
-- self-describing — copy the whole thing back to Claude for analysis.
--
-- All timestamps are TIMESTAMPTZ and the pool is pinned to UTC, so every
-- "day"/"last N days" window below is UTC. Read-only: no writes, no locks.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';  -- match the app's UTC-pinned pool so displayed times line up

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE USAGE OVERVIEW
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. SNAPSHOT — lifetime totals ──
-- One-line state of the world: who's signed up and how much they've done.
SELECT
    (SELECT count(*) FROM users)                                       AS users,
    (SELECT count(*) FROM users WHERE email_verified_at IS NOT NULL)   AS verified,
    (SELECT count(*) FROM users WHERE email_verified_at IS NULL)       AS unverified,
    (SELECT count(*) FROM decks)                                       AS decks,
    (SELECT count(*) FROM decks WHERE first_completed_at IS NOT NULL)  AS decks_completed,
    (SELECT COALESCE(sum(swipes_right + swipes_left + swipes_up + swipes_down), 0)
       FROM user_lifetime_counters)                                    AS total_swipes,
    (SELECT COALESCE(sum(searches), 0) FROM user_lifetime_counters)    AS total_searches;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. SIGNUPS — new users per day (last 21 days) ──
-- Are we growing, flat, or did a launch/marketing push move the needle?
SELECT
    created_at::date            AS day,
    count(*)                    AS signups,
    count(*) FILTER (WHERE email_verified_at IS NOT NULL) AS verified
FROM users
WHERE created_at >= CURRENT_DATE - INTERVAL '21 days'
GROUP BY 1
ORDER BY 1;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. ACTIVATION FUNNEL — signed up → swiped → built → completed ──
-- Durable-state funnel (counters + decks), so it's correct even for users
-- who signed up before event logging existed. Each step is a subset of the
-- one above it; the conv_% is share of all signups reaching that step.
WITH u AS (SELECT count(*)::numeric AS total FROM users)
SELECT step, users,
       round(100.0 * users / NULLIF((SELECT total FROM u), 0), 1) AS conv_pct
FROM (
    SELECT 1 AS ord, '1 signed up'        AS step, (SELECT count(*) FROM users) AS users
    UNION ALL
    SELECT 2, '2 swiped ≥1',  (SELECT count(*) FROM user_lifetime_counters
                               WHERE swipes_right + swipes_left + swipes_up + swipes_down > 0)
    UNION ALL
    SELECT 3, '3 searched ≥1', (SELECT count(*) FROM user_lifetime_counters WHERE searches > 0)
    UNION ALL
    SELECT 4, '4 built a deck', (SELECT count(DISTINCT user_id) FROM decks)
    UNION ALL
    SELECT 5, '5 completed deck', (SELECT count(DISTINCT user_id) FROM decks
                                   WHERE first_completed_at IS NOT NULL)
) f
ORDER BY ord;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. ENGAGEMENT — DAU / WAU / MAU + stickiness ──
-- Active = had any swipe/search telemetry that day (user_daily_activity).
-- DAU today is partial (the day isn't over). Stickiness = DAU/MAU.
SELECT
    count(*) FILTER (WHERE day = CURRENT_DATE)                       AS dau_today,
    count(*) FILTER (WHERE day = CURRENT_DATE - 1)                   AS dau_yesterday,
    count(*) FILTER (WHERE day >= CURRENT_DATE - 6)                  AS wau_7d,
    count(*) FILTER (WHERE day >= CURRENT_DATE - 29)                 AS mau_30d
FROM (SELECT DISTINCT user_id, day FROM user_daily_activity) a;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. RECENCY — when each user was last active ──
-- Buckets users by how long since their counters last moved
-- (user_lifetime_counters.updated_at doubles as last-active).
SELECT
    CASE
        WHEN updated_at >= now() - INTERVAL '1 day'  THEN '1 active <24h'
        WHEN updated_at >= now() - INTERVAL '7 days'  THEN '2 active <7d'
        WHEN updated_at >= now() - INTERVAL '30 days' THEN '3 active <30d'
        ELSE                                                '4 dormant 30d+'
    END                         AS recency,
    count(*)                    AS users
FROM user_lifetime_counters
GROUP BY 1
ORDER BY 1;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 6. DAILY ACTIVITY — active users + volume (last 14 days) ──
-- The pulse: distinct active users, total swipes, total searches per day.
SELECT
    day,
    count(DISTINCT user_id)                                     AS active_users,
    sum(swipes_right + swipes_left + swipes_up + swipes_down)   AS swipes,
    sum(searches)                                               AS searches
FROM user_daily_activity
WHERE day >= CURRENT_DATE - INTERVAL '14 days'
GROUP BY day
ORDER BY day;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 7. SWIPE BEHAVIOR — direction mix app-wide ──
-- right = primary action (add on search / remove on deck view), left = skip/pass,
-- up = maybeboard, down = undo. right:left is a proxy for how picky people are.
SELECT
    sum(swipes_right)                                                AS right_primary,
    sum(swipes_left)                                                 AS left_skip,
    sum(swipes_up)                                                   AS up_maybe,
    sum(swipes_down)                                                 AS down_undo,
    round(sum(swipes_right)::numeric
          / NULLIF(sum(swipes_left), 0), 3)                         AS right_per_left,
    sum(searches)                                                   AS searches
FROM user_lifetime_counters;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 8. ENGAGEMENT DISTRIBUTION — swipes-per-user histogram ──
-- Power users vs tire-kickers. How many users land in each volume bucket.
WITH per_user AS (
    SELECT swipes_right + swipes_left + swipes_up + swipes_down AS swipes
    FROM user_lifetime_counters
)
SELECT
    CASE
        WHEN swipes = 0          THEN '0 — never swiped'
        WHEN swipes < 10         THEN '1 — 1-9'
        WHEN swipes < 50         THEN '2 — 10-49'
        WHEN swipes < 200        THEN '3 — 50-199'
        WHEN swipes < 1000       THEN '4 — 200-999'
        ELSE                          '5 — 1000+'
    END                         AS swipe_bucket,
    count(*)                    AS users
FROM per_user
GROUP BY 1
ORDER BY 1;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 9. TOP USERS — most active by total swipes (top 15) ──
-- The people carrying the activity numbers. Username for recognizability.
SELECT
    u.username,
    c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down AS swipes,
    c.searches,
    c.decks_created,
    c.decks_completed,
    c.updated_at::date          AS last_active
FROM user_lifetime_counters c
JOIN users u ON u.id = c.user_id
ORDER BY swipes DESC, c.searches DESC
LIMIT 15;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 10. DECKS — per-user count distribution ──
-- Do people build one deck and stop, or keep going? (Includes 0-deck users.)
WITH per_user AS (
    SELECT u.id, count(d.id) AS decks
    FROM users u
    LEFT JOIN decks d ON d.user_id = u.id
    GROUP BY u.id
)
SELECT
    CASE
        WHEN decks = 0  THEN '0 decks'
        WHEN decks = 1  THEN '1 deck'
        WHEN decks <= 4 THEN '2-4 decks'
        ELSE                 '5+ decks'
    END                         AS deck_bucket,
    count(*)                    AS users
FROM per_user
GROUP BY 1
ORDER BY 1;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 11. DECK COMPLETION + SIZE ──
-- Completion rate = decks that ever reached a valid state. avg/median cards
-- counts the main board only (board='deck'), summing quantities.
WITH sizes AS (
    SELECT d.id, COALESCE(sum(dc.quantity) FILTER (WHERE dc.board = 'deck'), 0) AS cards
    FROM decks d
    LEFT JOIN deck_cards dc ON dc.deck_id = d.id
    GROUP BY d.id
)
SELECT
    (SELECT count(*) FROM decks)                                      AS decks,
    (SELECT count(*) FROM decks WHERE first_completed_at IS NOT NULL) AS completed,
    round(100.0 * (SELECT count(*) FROM decks WHERE first_completed_at IS NOT NULL)
          / NULLIF((SELECT count(*) FROM decks), 0), 1)              AS completion_pct,
    round(avg(cards), 1)                                             AS avg_cards,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY cards)               AS median_cards,
    max(cards)                                                       AS max_cards
FROM sizes;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 12. FORMAT POPULARITY ──
-- Which formats people actually build in. NULL = format not set on the deck.
SELECT
    COALESCE(format, '· (unset)') AS format,
    count(*)                      AS decks,
    count(*) FILTER (WHERE first_completed_at IS NOT NULL) AS completed
FROM decks
GROUP BY format
ORDER BY decks DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 13. TOP COMMANDERS (top 20) ──
-- Most-chosen commanders across all decks. Tells you the metagame in-app.
SELECT
    sd.name                     AS commander,
    count(*)                    AS decks
FROM decks d
JOIN scryfall_data sd ON sd.id = d.commander_id
WHERE d.commander_id IS NOT NULL
GROUP BY sd.name
ORDER BY decks DESC
LIMIT 20;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 14. TOP CARDS ADDED (main board, top 25) ──
-- Most-included cards across decks — staples people reach for. Counts
-- distinct decks the card appears in (board='deck').
SELECT
    min(sd.name)                AS card,
    count(DISTINCT dc.deck_id)  AS in_decks,
    sum(dc.quantity)            AS total_copies
FROM deck_cards dc
JOIN scryfall_data sd ON sd.id = dc.scryfall_data_id
WHERE dc.board = 'deck'
GROUP BY dc.oracle_id
ORDER BY in_decks DESC, total_copies DESC
LIMIT 25;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 15. BOARD USAGE — deck vs maybeboard vs sideboard ──
-- Are people using the maybeboard (swipe-up) and sideboard features at all?
SELECT
    board,
    count(*)                    AS card_rows,
    count(DISTINCT deck_id)     AS decks_using,
    sum(quantity)               AS total_copies
FROM deck_cards
GROUP BY board
ORDER BY card_rows DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 16. ACCOUNT ACTIVITY — credential changes + lockouts ──
-- Identity churn (from the audit log) and any accounts currently locked out.
SELECT action, count(*) AS events
FROM user_audit_log
GROUP BY action
ORDER BY events DESC;

\echo
SELECT
    count(*) FILTER (WHERE lockout_until > now())     AS currently_locked_out,
    count(*) FILTER (WHERE failed_login_attempts > 0) AS users_with_failed_logins
FROM users;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF REPORT — copy everything above back for analysis
\echo ════════════════════════════════════════════════════════════════
