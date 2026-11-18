-- Public lifetime totals check: why would the totals on Home go down?
--
-- The totals are SUM(user_lifetime_counters) plus the one retired_lifetime_counters row, which
-- a BEFORE DELETE trigger fills as accounts go, so they should never drop. This prints what the
-- API computes now, the parts it adds up, whether the trigger is in place, the tables' write
-- statistics since the last stats reset, the biggest accounts, and every counter row that is
-- lower than the account's own decks and daily activity say it must be.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/totals_check.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/totals_check.sql
--
-- Read-only: one READ ONLY transaction with a statement timeout, SELECTs only.

\set ON_ERROR_STOP on
\pset border 2
\pset null '·'
\timing off

BEGIN TRANSACTION READ ONLY;
SET LOCAL statement_timeout = '30s';
SET LOCAL TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE PUBLIC TOTALS CHECK
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc,
       pg_postmaster_start_time() AS server_started_at,
       (SELECT stats_reset FROM pg_stat_database WHERE datname = current_database()) AS stats_reset_at;

\echo
\echo ── 1. Public totals, as GET /api/marketing/stats computes them ──
SELECT
    (COALESCE(SUM(u.swipes_right + u.swipes_left + u.swipes_up + u.swipes_down), 0)
        + r.swipes_right + r.swipes_left + r.swipes_up + r.swipes_down)::BIGINT AS cards_swiped,
    (COALESCE(SUM(u.searches), 0) + r.searches)::BIGINT AS searches,
    (COALESCE(SUM(u.decks_created), 0) + r.decks_created)::BIGINT AS decks_created
FROM retired_lifetime_counters r
LEFT JOIN user_lifetime_counters u ON TRUE
GROUP BY r.id;

\echo
\echo ── 2. The two parts: counters of live accounts, and the retired row ──
SELECT 'live accounts' AS part,
       count(*) AS rows,
       COALESCE(SUM(swipes_right + swipes_left + swipes_up + swipes_down), 0) AS swipes,
       COALESCE(SUM(searches), 0) AS searches,
       COALESCE(SUM(decks_created), 0) AS decks_created,
       COALESCE(SUM(decks_completed), 0) AS decks_completed
FROM user_lifetime_counters
UNION ALL
SELECT 'retired (deleted accounts)', 1,
       swipes_right + swipes_left + swipes_up + swipes_down, searches, decks_created, decks_completed
FROM retired_lifetime_counters;

\echo
\echo ── 3. The retire trigger: present and enabled? (tgenabled O = on, D = disabled) ──
-- A TRUNCATE, or a session with session_replication_role = replica, skips it.
SELECT t.tgname, c.relname AS on_table, t.tgenabled, p.proname AS function
FROM pg_trigger t
JOIN pg_class c ON c.oid = t.tgrelid
JOIN pg_proc p ON p.oid = t.tgfoid
WHERE NOT t.tgisinternal
  AND c.relname IN ('user_lifetime_counters', 'retired_lifetime_counters', 'users');

\echo
\echo ── 4. Row counts the totals lean on ──
SELECT
    (SELECT count(*) FROM users) AS users,
    (SELECT count(*) FROM user_lifetime_counters) AS counter_rows,
    (SELECT count(*) FROM users u WHERE NOT EXISTS
        (SELECT 1 FROM user_lifetime_counters c WHERE c.user_id = u.id)) AS users_without_counters,
    (SELECT count(*) FROM decks) AS decks_now,
    (SELECT count(*) FROM user_events WHERE kind = 'deck_created') AS deck_created_events;

\echo
\echo ── 5. Write statistics since the stats reset (inserts, updates, deletes per table) ──
-- n_tup_del on users counts account deletions; a drop in n_live_tup with no matching
-- deletes points at a TRUNCATE. Statistics start over at a server restart after a crash.
SELECT relname, n_live_tup, n_tup_ins, n_tup_upd, n_tup_hot_upd, n_tup_del,
       last_autovacuum, last_autoanalyze
FROM pg_stat_user_tables
WHERE relname IN ('user_lifetime_counters', 'retired_lifetime_counters', 'users', 'decks',
                  'user_daily_activity')
ORDER BY relname;

\echo
\echo ── 6. Top 20 accounts by decks_created ──
SELECT u.username, c.decks_created, c.decks_completed,
       (SELECT count(*) FROM decks d WHERE d.user_id = u.id) AS decks_now,
       c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down AS swipes,
       u.created_at, c.updated_at AS counters_updated_at
FROM user_lifetime_counters c
JOIN users u ON u.id = c.user_id
ORDER BY c.decks_created DESC
LIMIT 20;

\echo
\echo ── 7. Top 20 accounts by swipes ──
SELECT u.username,
       c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down AS swipes,
       (SELECT COALESCE(SUM(a.swipes_right + a.swipes_left + a.swipes_up + a.swipes_down), 0)
          FROM user_daily_activity a WHERE a.user_id = u.id) AS daily_swipes,
       c.searches, c.decks_created,
       u.created_at, c.updated_at AS counters_updated_at
FROM user_lifetime_counters c
JOIN users u ON u.id = c.user_id
ORDER BY swipes DESC
LIMIT 20;

\echo
\echo ── 8. Counter rows lower than the records of their own account say they must be ──
-- Each floor only counts things the counters also count, so a counter under its floor was
-- lowered: decks_created under its deck_created events; swipes or searches under the sum of its
-- daily activity. Rows updated in the last 48 hours first. decks_now is for reference only:
-- decks made before the counters began, or by a script, can leave it above decks_created.
WITH floors AS (
    SELECT c.user_id,
           c.decks_created,
           (SELECT count(*) FROM decks d WHERE d.user_id = c.user_id) AS decks_now,
           (SELECT count(*) FROM user_events e
              WHERE e.user_id = c.user_id AND e.kind = 'deck_created') AS deck_events,
           c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down AS swipes,
           c.searches,
           a.daily_swipes,
           a.daily_searches,
           c.updated_at
    FROM user_lifetime_counters c
    LEFT JOIN LATERAL (
        SELECT COALESCE(SUM(swipes_right + swipes_left + swipes_up + swipes_down), 0) AS daily_swipes,
               COALESCE(SUM(searches), 0) AS daily_searches
        FROM user_daily_activity WHERE user_id = c.user_id
    ) a ON TRUE
)
SELECT u.username,
       f.updated_at >= now() - interval '48 hours' AS updated_last_48h,
       f.updated_at AS counters_updated_at,
       f.decks_created, f.decks_now, f.deck_events,
       f.swipes, f.daily_swipes,
       f.searches, f.daily_searches
FROM floors f
JOIN users u ON u.id = f.user_id
WHERE f.decks_created < f.deck_events
   OR f.swipes < f.daily_swipes
   OR f.searches < f.daily_searches
ORDER BY updated_last_48h DESC, f.updated_at DESC
LIMIT 100;

\echo
\echo ── 9. How many rows are under a floor, all time and in the last 48 hours ──
WITH floors AS (
    SELECT c.updated_at,
           c.decks_created < (SELECT count(*) FROM user_events e
                                    WHERE e.user_id = c.user_id AND e.kind = 'deck_created')
             OR c.swipes_right + c.swipes_left + c.swipes_up + c.swipes_down
                < (SELECT COALESCE(SUM(swipes_right + swipes_left + swipes_up + swipes_down), 0)
                     FROM user_daily_activity a WHERE a.user_id = c.user_id)
             OR c.searches < (SELECT COALESCE(SUM(searches), 0)
                                FROM user_daily_activity a WHERE a.user_id = c.user_id) AS low
    FROM user_lifetime_counters c
)
SELECT count(*) FILTER (WHERE low) AS under_a_floor,
       count(*) FILTER (WHERE low AND updated_at >= now() - interval '48 hours') AS under_a_floor_last_48h,
       count(*) FILTER (WHERE updated_at >= now() - interval '48 hours') AS rows_updated_last_48h
FROM floors;

\echo
\echo ── 10. Account deletions ──
-- Nothing records a deletion: the account's rows go with it. The signals are users.n_tup_del
-- in section 5 and the retired row in section 2. Accounts created in the last 7 days that still
-- exist, by day, for comparison with registrations seen elsewhere:
SELECT created_at::date AS day, count(*) AS accounts_still_here
FROM users
WHERE created_at >= now() - interval '7 days'
GROUP BY 1
ORDER BY 1;

\echo
\echo ── 11. Daily activity, last 7 days (all accounts still here) ──
SELECT day,
       SUM(swipes_right + swipes_left + swipes_up + swipes_down) AS swipes,
       SUM(searches) AS searches,
       count(*) AS active_accounts
FROM user_daily_activity
WHERE day >= current_date - 7
GROUP BY day
ORDER BY day;

COMMIT;
