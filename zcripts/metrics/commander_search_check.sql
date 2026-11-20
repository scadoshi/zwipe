-- Commander search check: why would search/commanders hit its 10s statement timeout under load?
--
-- The commander swipe stack's search ranks every commander-legal printing (about 3,500) on each
-- page it serves. Warm, the ranking takes a fraction of a second; read cold from disk it takes
-- seconds, so a cache that misses under load is the likely reason for a timeout. This prints the
-- memory settings, how often the catalog's tables are read from the cache, how long the search
-- takes as pg_stat_statements has timed it, and what is running right now.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/commander_search_check.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/commander_search_check.sql
-- The sudo -u postgres form is the one that counts everything in section 5: run as the app role,
-- without pg_read_all_stats, other roles' rows show no state there, so its counts read low.
--
-- Read-only: one READ ONLY transaction with a statement timeout. No user id, name or query
-- parameter in the output, only settings, counts and times. A statement the timeout cancels
-- never reaches pg_stat_statements, so section 4 times the searches that finished; zerver's log
-- names the cancelled ones ("canceling statement due to statement timeout").

\set ON_ERROR_STOP on
\set QUIET on
\pset border 2
\pset null '·'
\timing off

BEGIN TRANSACTION READ ONLY;
SET LOCAL statement_timeout = '120s';
SET LOCAL TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE COMMANDER SEARCH CHECK
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc, version() AS server;

\echo
\echo ── 1. Memory and timeout settings ──
-- shared_buffers is the cache the catalog has to fit in; work_mem is what a sort gets before it
-- spills to disk. The search sets its own work_mem, transaction-local.
SELECT name, setting, unit, source
FROM pg_settings
WHERE name IN ('shared_buffers', 'effective_cache_size', 'work_mem', 'statement_timeout',
               'max_connections', 'random_page_cost', 'jit')
ORDER BY name;

\echo
\echo ── 2. Cache hits: the database, then the tables the search reads ──
-- hit_pct near 100 means reads come from memory. A low number on latest_cards or card_profiles
-- is the cold read that makes the ranking slow. Counts are since the statistics were last reset.
SELECT
    datname AS database,
    blks_hit,
    blks_read,
    round(100.0 * blks_hit / NULLIF(blks_hit + blks_read, 0), 2) AS hit_pct,
    temp_files,
    pg_size_pretty(temp_bytes) AS temp_written,
    stats_reset
FROM pg_stat_database
WHERE datname = current_database();

SELECT
    relname AS relation,
    pg_size_pretty(pg_total_relation_size(relid)) AS size,
    heap_blks_hit,
    heap_blks_read,
    round(100.0 * heap_blks_hit / NULLIF(heap_blks_hit + heap_blks_read, 0), 2) AS heap_hit_pct,
    idx_blks_hit,
    idx_blks_read,
    round(100.0 * idx_blks_hit / NULLIF(idx_blks_hit + idx_blks_read, 0), 2) AS idx_hit_pct
FROM pg_statio_all_tables
WHERE relname IN ('latest_cards', 'card_profiles', 'commander_popularity',
                  'commander_select_signal', 'scryfall_data')
ORDER BY pg_total_relation_size(relid) DESC;

\echo
\echo ── 3. The pool the search ranks ──
-- The commander-legal printings the swipe stack's first page ranks, as the search narrows them.
SELECT count(*) AS commander_pool, pg_size_pretty(sum(pg_column_size(lc.*))::bigint) AS pool_bytes
FROM latest_cards lc
JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
WHERE lc.layout NOT IN ('token', 'double_faced_token', 'emblem')
  AND NOT cp.is_token
  AND lc.legalities->>'commander' IN ('legal', 'restricted')
  AND ((lc.type_line ILIKE '%Legendary%' AND split_part(lc.type_line, ' // ', 1) ILIKE '%Legendary%'
        AND ((lc.type_line ILIKE '%Creature%' AND split_part(lc.type_line, ' // ', 1) ILIKE '%Creature%')
             OR (lc.power IS NOT NULL AND lc.toughness IS NOT NULL)))
       OR lc.oracle_text ILIKE '%can be your commander%');

\echo
\echo ── 4. How long the search takes, from pg_stat_statements ──
SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'pg_stat_statements') AS has_pss \gset
\if :has_pss
-- One row per statement shape. The commander pool's ranking starts "WITH pool AS (SELECT
-- latest_cards" and joins commander_popularity. Times are milliseconds; temp_written is what
-- its sorts spilled to disk. Only the statement's first words print, never its parameters.
SELECT
    left(regexp_replace(query, '\s+', ' ', 'g'), 70) AS statement,
    calls,
    round(mean_exec_time::numeric, 1) AS mean_ms,
    round(max_exec_time::numeric, 1) AS max_ms,
    round(stddev_exec_time::numeric, 1) AS stddev_ms,
    round(100.0 * shared_blks_hit / NULLIF(shared_blks_hit + shared_blks_read, 0), 2) AS hit_pct,
    pg_size_pretty(temp_blks_written * current_setting('block_size')::bigint) AS temp_written
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = current_database())
  AND query LIKE 'WITH pool AS (SELECT latest_cards%'
  AND query LIKE '%commander_popularity%'
ORDER BY max_exec_time DESC
LIMIT 10;
\else
\echo pg_stat_statements is not installed: no times to read. It needs
\echo shared_preload_libraries set to pg_stat_statements and CREATE EXTENSION pg_stat_statements.
\endif

\echo
\echo ── 5. Right now: what is running ──
-- How many statements are active and waiting, and how many of them are a commander search.
-- No query text, user or address prints.
SELECT
    count(*) FILTER (WHERE state = 'active') AS active,
    count(*) FILTER (WHERE state = 'active' AND wait_event IS NOT NULL) AS active_waiting,
    count(*) FILTER (WHERE state = 'active' AND query LIKE 'WITH pool AS (SELECT latest_cards%'
                     AND query LIKE '%commander_popularity%') AS commander_searches,
    count(*) FILTER (WHERE state = 'idle in transaction') AS idle_in_transaction,
    max(now() - query_start) FILTER (WHERE state = 'active') AS longest_active
FROM pg_stat_activity
WHERE datname = current_database()
  AND pid <> pg_backend_pid();

COMMIT;
