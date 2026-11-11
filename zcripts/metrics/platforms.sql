-- Zwipe platforms: "who is on iOS, Android and the web?"
--
-- Users, new users, usage and the signup funnel split by client platform
-- (ios, android, web, desktop, terminal; unknown is an older client or a
-- token minted without one).
--
--   psql "$DATABASE_URL" -f zcripts/metrics/platforms.sql
--   sudo -u postgres psql zwipe -f zcripts/metrics/platforms.sql
--
-- Read-only. All windows are UTC (pool is pinned to UTC); today is partial.
-- Prints counts only, no user ids or names.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE PLATFORMS
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. USERS BY PLATFORM — last 30 days ──
-- From sessions (refresh_tokens.platform). A user signed in on two
-- platforms counts in both.
SELECT coalesce(platform, 'unknown') AS platform,
       count(DISTINCT user_id)       AS users
FROM refresh_tokens
WHERE created_at > now() - interval '30 days'
GROUP BY 1
ORDER BY 2 DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. USERS BY PLATFORM — last 7 days ──
SELECT coalesce(platform, 'unknown') AS platform,
       count(DISTINCT user_id)       AS users
FROM refresh_tokens
WHERE created_at > now() - interval '7 days'
GROUP BY 1
ORDER BY 2 DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. NEW USERS BY FIRST PLATFORM — last 7 days ──
-- Each user registered in the window, counted once, by the platform of
-- their earliest session.
SELECT coalesce(first_session.platform, 'unknown') AS platform,
       count(*)                                    AS new_users
FROM users u
JOIN LATERAL (
    SELECT r.platform
    FROM refresh_tokens r
    WHERE r.user_id = u.id
    ORDER BY r.created_at
    LIMIT 1
) first_session ON true
WHERE u.created_at > now() - interval '7 days'
GROUP BY 1
ORDER BY 2 DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. SWIPES + SEARCHES BY PLATFORM PER DAY — last 14 days ──
-- From platform_daily_usage (no user ids).
SELECT day,
       platform,
       swipes_right + swipes_left + swipes_up + swipes_down AS swipes,
       searches
FROM platform_daily_usage
WHERE day > current_date - 14
ORDER BY day, platform;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. SIGNUP FUNNEL BY PLATFORM — last 7 days (distinct sessions) ──
-- anonymous_events.platform; null is a client older than the column.
SELECT coalesce(platform, 'unknown') AS platform,
       kind,
       count(DISTINCT session_id)    AS sessions
FROM anonymous_events
WHERE occurred_at > now() - interval '7 days'
GROUP BY 1, 2
ORDER BY 1, 2;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF PLATFORMS
\echo ════════════════════════════════════════════════════════════════
