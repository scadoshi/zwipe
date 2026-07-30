-- Client error + crash report reader — the field-diagnostics view.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/errors.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/errors.sql
--
-- Reads the two tables fed by the 1.7.4+ clients: `client_errors` (handled
-- errors shown to users, deduped per flush window with breadcrumbs) and
-- `crash_reports` (panics, exactly-once per crash_id). Both are anonymous —
-- there is no user column to join. Rows age out at 90 days via the zervice
-- upkeep prune, so every "all time" here really means "last 90 days".
--
-- The one alarm to watch: a DECODE spike on a single client_version = wire
-- contract drift between that build and the server. Section 4 exists for it.
--
-- Read-only: no writes, no locks.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE CLIENT ERRORS + CRASHES
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. SNAPSHOT — how much is coming in ──
-- Totals within the retention window, and whether anything arrived recently.
SELECT
    (SELECT count(*) FROM client_errors)                          AS error_rows,
    (SELECT COALESCE(sum(count), 0) FROM client_errors)           AS error_occurrences,
    (SELECT max(received_at) FROM client_errors)                  AS last_error_at,
    (SELECT count(*) FROM crash_reports)                          AS crashes,
    (SELECT max(received_at) FROM crash_reports)                  AS last_crash_at;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. ERRORS BY DAY — last 14 days, version × screen × kind ──
-- The daily shape. `occurrences` sums the client-side dedupe counts, so it is
-- the real how-often; `rows` is how many distinct flush-window entries landed.
SELECT
    received_at::date            AS day,
    client_version               AS version,
    screen,
    kind,
    count(*)                     AS rows,
    sum(count)                   AS occurrences
FROM client_errors
WHERE received_at > now() - INTERVAL '14 days'
GROUP BY 1, 2, 3, 4
ORDER BY 1 DESC, occurrences DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. TOP ERROR SIGNATURES — what hurts users most ──
-- Full breadcrumb + message, ranked by total occurrences. `component` is
-- blank when the screen itself surfaced the error.
SELECT
    screen,
    NULLIF(component, '')        AS component,
    action,
    kind,
    left(message, 70)            AS message,
    count(*)                     AS rows,
    sum(count)                   AS occurrences,
    min(received_at)::date       AS first_seen,
    max(received_at)::date       AS last_seen,
    count(DISTINCT client_version) AS versions
FROM client_errors
GROUP BY screen, component, action, kind, message
ORDER BY occurrences DESC
LIMIT 25;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. CONTRACT-DRIFT ALARM — decode + server-error kinds by version ──
-- decode on one version = that build disagrees with the server about the
-- wire shape. api_internal = the server 500'd at a user. Both deserve a look
-- whenever nonzero.
SELECT
    client_version               AS version,
    platform,
    kind,
    count(*)                     AS rows,
    sum(count)                   AS occurrences,
    max(received_at)             AS last_seen
FROM client_errors
WHERE kind IN ('decode', 'api_internal')
GROUP BY 1, 2, 3
ORDER BY last_seen DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. CRASHES — grouped by version and panic site ──
-- The message's first line carries file:line:column, so grouping by its
-- prefix clusters identical panic sites across users.
SELECT
    client_version               AS version,
    platform,
    left(split_part(message, chr(10), 1), 80) AS panic_site,
    count(*)                     AS crashes,
    min(received_at)::date       AS first_seen,
    max(received_at)::date       AS last_seen
FROM crash_reports
GROUP BY 1, 2, 3
ORDER BY crashes DESC, last_seen DESC;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 6. RECENT RAW TAIL — the latest of each, verbatim ──
-- For immediate eyes-on reading after a release.
SELECT
    received_at,
    client_version               AS version,
    platform,
    screen,
    NULLIF(component, '')        AS component,
    action,
    kind,
    left(message, 60)            AS message,
    count
FROM client_errors
ORDER BY received_at DESC
LIMIT 15;

SELECT
    received_at,
    occurred_at,
    client_version               AS version,
    platform,
    left(message, 110)           AS message
FROM crash_reports
ORDER BY received_at DESC
LIMIT 10;
