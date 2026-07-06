-- Zwipe pulse — "what happened in the last couple of days?"
--
-- The quick check between full overview runs: today vs yesterday, the last
-- 7 days, who just registered, and what the event log saw recently.
--
--   psql "$DATABASE_URL" -f zcripts/metrics/pulse.sql
--   sudo -u postgres psql zwipe -f zcripts/metrics/pulse.sql
--
-- Read-only. All windows are UTC (pool is pinned to UTC); today is partial.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE PULSE
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. TODAY VS YESTERDAY ──
-- Registrations from users.created_at; activity from user_daily_activity.
WITH act AS (
    SELECT day,
           count(DISTINCT user_id)                                   AS active_users,
           sum(swipes_right + swipes_left + swipes_up + swipes_down) AS swipes,
           sum(searches)                                             AS searches
    FROM user_daily_activity
    WHERE day >= CURRENT_DATE - 1
    GROUP BY day
), reg AS (
    SELECT created_at::date AS day, count(*) AS registrations
    FROM users
    WHERE created_at >= CURRENT_DATE - 1
    GROUP BY 1
)
SELECT d.day,
       COALESCE(r.registrations, 0) AS registrations,
       COALESCE(a.active_users, 0)  AS active_users,
       COALESCE(a.swipes, 0)        AS swipes,
       COALESCE(a.searches, 0)      AS searches
FROM (SELECT CURRENT_DATE AS day UNION ALL SELECT CURRENT_DATE - 1) d
LEFT JOIN act a ON a.day = d.day
LEFT JOIN reg r ON r.day = d.day
ORDER BY d.day DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. LAST 7 DAYS — registrations + activity per day ──
WITH act AS (
    SELECT day,
           count(DISTINCT user_id)                                   AS active_users,
           sum(swipes_right + swipes_left + swipes_up + swipes_down) AS swipes,
           sum(searches)                                             AS searches
    FROM user_daily_activity
    WHERE day >= CURRENT_DATE - 6
    GROUP BY day
), reg AS (
    SELECT created_at::date AS day, count(*) AS registrations
    FROM users
    WHERE created_at >= CURRENT_DATE - 6
    GROUP BY 1
)
SELECT d.day::date                  AS day,
       COALESCE(r.registrations, 0) AS registrations,
       COALESCE(a.active_users, 0)  AS active_users,
       COALESCE(a.swipes, 0)        AS swipes,
       COALESCE(a.searches, 0)      AS searches
FROM generate_series(CURRENT_DATE - 6, CURRENT_DATE, '1 day') d(day)
LEFT JOIN act a ON a.day = d.day::date
LEFT JOIN reg r ON r.day = d.day::date
ORDER BY d.day;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. LATEST REGISTRATIONS (last 10) — did they stick? ──
-- Each new account with what they've done since: verified, swiped, built.
SELECT
    u.username,
    u.created_at::date                                            AS registered,
    (u.email_verified_at IS NOT NULL)                             AS verified,
    COALESCE(c.swipes_right + c.swipes_left
             + c.swipes_up + c.swipes_down, 0)                    AS swipes,
    COALESCE(c.searches, 0)                                       AS searches,
    COALESCE(c.decks_created, 0)                                  AS decks,
    u.last_active_at::date                                        AS last_active
FROM users u
LEFT JOIN user_lifetime_counters c ON c.user_id = u.id
ORDER BY u.created_at DESC
LIMIT 10;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. EVENT LOG — last 48h by kind ──
-- Sparse durable events: register, login, refresh, logout, deck_created,
-- deck_completed, first_swipe.
SELECT kind, count(*) AS events, count(DISTINCT user_id) AS users
FROM user_events
WHERE occurred_at >= now() - INTERVAL '48 hours'
GROUP BY kind
ORDER BY events DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. PRE-AUTH FUNNEL — last 48h (sessions per step) ──
-- Distinct anonymous sessions reaching each step. Compare against the
-- register events above; funnel.sql has the full breakdown.
-- (Empty until the anonymous_events migration + client are live.)
SELECT kind, count(DISTINCT session_id) AS sessions, count(*) AS events
FROM anonymous_events
WHERE occurred_at >= now() - INTERVAL '48 hours'
GROUP BY kind
ORDER BY CASE kind
    WHEN 'app_opened'         THEN 1
    WHEN 'register_viewed'    THEN 2
    WHEN 'register_submitted' THEN 3
    ELSE 4 END;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF PULSE
\echo ════════════════════════════════════════════════════════════════
