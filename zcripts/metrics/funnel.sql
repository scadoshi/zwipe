-- Zwipe pre-auth funnel — app opened → register viewed → submitted → registered.
--
-- Built on anonymous_events (client-generated session UUIDs, no identity) plus
-- registrations from users.created_at. Sessions can't be joined to accounts by
-- design, so the final step is a count alongside, not a per-session join.
--
--   psql "$DATABASE_URL" -f zcripts/metrics/funnel.sql
--   sudo -u postgres psql zwipe -f zcripts/metrics/funnel.sql
--
-- Read-only. UTC windows. Empty until the anonymous_events migration and the
-- instrumented client build are both live — a low app_opened count just means
-- most installs are still on older builds.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE PRE-AUTH FUNNEL
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. FUNNEL — last 30 days ──
-- Distinct sessions per step, each step's share of app_opened, and
-- registrations (from users) over the same window for the final leg.
WITH steps AS (
    SELECT kind, count(DISTINCT session_id) AS sessions
    FROM anonymous_events
    WHERE occurred_at >= now() - INTERVAL '30 days'
    GROUP BY kind
), opened AS (
    SELECT COALESCE((SELECT sessions FROM steps WHERE kind = 'app_opened'), 0)::numeric AS n
)
SELECT step, sessions,
       round(100.0 * sessions / NULLIF((SELECT n FROM opened), 0), 1) AS pct_of_opened
FROM (
    SELECT 1 AS ord, '1 app_opened' AS step,
           COALESCE((SELECT sessions FROM steps WHERE kind = 'app_opened'), 0) AS sessions
    UNION ALL
    SELECT 2, '2 register_viewed',
           COALESCE((SELECT sessions FROM steps WHERE kind = 'register_viewed'), 0)
    UNION ALL
    SELECT 3, '3 register_submitted',
           COALESCE((SELECT sessions FROM steps WHERE kind = 'register_submitted'), 0)
    UNION ALL
    SELECT 4, '4 registered (accounts)',
           (SELECT count(*) FROM users WHERE created_at >= now() - INTERVAL '30 days')
) f
ORDER BY ord;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. DAILY FUNNEL — last 14 days ──
-- Sessions per step per day, with that day's registrations alongside.
WITH s AS (
    SELECT occurred_at::date AS day, kind, session_id
    FROM anonymous_events
    WHERE occurred_at >= CURRENT_DATE - INTERVAL '14 days'
), reg AS (
    SELECT created_at::date AS day, count(*) AS registered
    FROM users
    WHERE created_at >= CURRENT_DATE - INTERVAL '14 days'
    GROUP BY 1
)
SELECT
    d.day,
    count(DISTINCT s.session_id) FILTER (WHERE s.kind = 'app_opened')         AS opened,
    count(DISTINCT s.session_id) FILTER (WHERE s.kind = 'register_viewed')    AS viewed,
    count(DISTINCT s.session_id) FILTER (WHERE s.kind = 'register_submitted') AS submitted,
    COALESCE(max(reg.registered), 0)                                          AS registered
FROM (SELECT DISTINCT day FROM s UNION SELECT day FROM reg) d
LEFT JOIN s   ON s.day = d.day
LEFT JOIN reg ON reg.day = d.day
GROUP BY d.day
ORDER BY d.day;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. FRICTION — time from open to submit (sessions that got there) ──
-- Per-session gap between first app_opened and first register_submitted.
-- A long median means the register flow is making people think too hard.
WITH per_session AS (
    SELECT session_id,
           min(occurred_at) FILTER (WHERE kind = 'app_opened')         AS opened_at,
           min(occurred_at) FILTER (WHERE kind = 'register_submitted') AS submitted_at
    FROM anonymous_events
    WHERE occurred_at >= now() - INTERVAL '30 days'
    GROUP BY session_id
)
SELECT
    count(*)                                                            AS sessions_submitted,
    round(EXTRACT(EPOCH FROM percentile_cont(0.5)
          WITHIN GROUP (ORDER BY submitted_at - opened_at))::numeric)   AS median_seconds,
    round(EXTRACT(EPOCH FROM percentile_cont(0.9)
          WITHIN GROUP (ORDER BY submitted_at - opened_at))::numeric)   AS p90_seconds
FROM per_session
WHERE opened_at IS NOT NULL AND submitted_at IS NOT NULL;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. RETRIES — submit attempts per submitting session ──
-- >1 submit in a session = validation errors or failed attempts before
-- success (or giving up). A high multi-submit share means form friction.
WITH submits AS (
    SELECT session_id, count(*) AS attempts
    FROM anonymous_events
    WHERE kind = 'register_submitted'
      AND occurred_at >= now() - INTERVAL '30 days'
    GROUP BY session_id
)
SELECT
    count(*)                                    AS submitting_sessions,
    count(*) FILTER (WHERE attempts = 1)        AS one_attempt,
    count(*) FILTER (WHERE attempts BETWEEN 2 AND 3) AS two_to_three,
    count(*) FILTER (WHERE attempts > 3)        AS four_plus,
    max(attempts)                               AS max_attempts
FROM submits;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF FUNNEL
\echo ════════════════════════════════════════════════════════════════
