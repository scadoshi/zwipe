-- Card signal size: what would per-card swipe numbers cost to store and read?
--
-- For the card screen's metrics (zwipe-context plans/card_lookup_metrics.md). Prints the swipe
-- signal tables' row counts and sizes on disk, how many rows each card has, how often the tables
-- are written, and estimates for the two new structures the plan weighs: a per-card table
-- (card_swipe_signal) and an index on commander_card_signal (card_oracle_id). The timed sections
-- show what a nightly rollup's aggregates cost.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/card_signal_size.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/card_signal_size.sql > /tmp/card-signal-size.txt
--
-- Read-only: one READ ONLY transaction with a statement timeout, SELECTs only. No account is
-- named; cards show by their public name.

\set ON_ERROR_STOP on
\pset border 2
\pset null '·'
\timing off

BEGIN TRANSACTION READ ONLY;
SET LOCAL statement_timeout = '60s';
SET LOCAL TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE CARD SIGNAL SIZE
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc,
       pg_postmaster_start_time() AS server_started_at,
       (SELECT stats_reset FROM pg_stat_database WHERE datname = current_database()) AS stats_reset_at;

\echo
\echo ── 1. Tables and views: estimated rows and size on disk ──
SELECT c.relname AS relation,
       CASE c.relkind WHEN 'r' THEN 'table' WHEN 'm' THEN 'matview' END AS kind,
       c.reltuples::BIGINT AS est_rows,
       pg_size_pretty(pg_relation_size(c.oid)) AS heap,
       pg_size_pretty(pg_indexes_size(c.oid)) AS indexes,
       pg_size_pretty(pg_total_relation_size(c.oid)) AS total,
       pg_total_relation_size(c.oid) AS total_bytes
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = 'public'
WHERE c.relname IN ('commander_card_signal', 'user_card_signal', 'card_signal_rollup',
                    'commander_select_signal', 'otag_context_signal', 'otag_context_signal_rollup',
                    'user_week_signal', 'usage_signal_budget', 'deck_card_suppressions',
                    'deck_cards', 'decks', 'latest_cards')
ORDER BY pg_total_relation_size(c.oid) DESC;

\echo
\echo ── 2. Each index on the signal tables and deck_cards ──
SELECT i.relname AS index,
       t.relname AS on_table,
       pg_size_pretty(pg_relation_size(i.oid)) AS size,
       pg_relation_size(i.oid) AS bytes,
       pg_get_indexdef(i.oid) AS definition
FROM pg_index x
JOIN pg_class i ON i.oid = x.indexrelid
JOIN pg_class t ON t.oid = x.indrelid
WHERE t.relname IN ('commander_card_signal', 'user_card_signal', 'card_signal_rollup',
                    'deck_cards', 'decks')
ORDER BY t.relname, i.relname;

\timing on

\echo
\echo ── 3. commander_card_signal: exact rows, cards, commanders, and swipes (timed: a rollup scans this) ──
SELECT count(*) AS rows,
       count(DISTINCT card_oracle_id) AS cards,
       count(DISTINCT commander_oracle_id) AS commanders,
       sum(shown) AS shown, sum(added) AS added, sum(skipped) AS skipped,
       sum(maybed) AS maybed, sum(removed) AS removed
FROM commander_card_signal;

\echo
\echo ── 4. commander_card_signal: rows per card (how many commanders each card has a row under) ──
WITH per_card AS (
    SELECT card_oracle_id, count(*) AS n, sum(shown) AS shown
    FROM commander_card_signal
    GROUP BY card_oracle_id
)
SELECT count(*) AS cards,
       percentile_disc(0.5) WITHIN GROUP (ORDER BY n) AS p50_rows,
       percentile_disc(0.9) WITHIN GROUP (ORDER BY n) AS p90_rows,
       percentile_disc(0.99) WITHIN GROUP (ORDER BY n) AS p99_rows,
       max(n) AS max_rows,
       percentile_disc(0.5) WITHIN GROUP (ORDER BY shown) AS p50_shown,
       percentile_disc(0.9) WITHIN GROUP (ORDER BY shown) AS p90_shown,
       max(shown) AS max_shown
FROM per_card;

\echo
\echo ── 5. The 10 cards with the most commander rows ──
SELECT lc.name, s.n AS rows, s.shown, s.added, s.skipped, s.removed
FROM (
    SELECT card_oracle_id, count(*) AS n, sum(shown) AS shown, sum(added) AS added,
           sum(skipped) AS skipped, sum(removed) AS removed
    FROM commander_card_signal
    GROUP BY card_oracle_id
    ORDER BY count(*) DESC
    LIMIT 10
) s
LEFT JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
ORDER BY s.n DESC;

\echo
\echo ── 6. user_card_signal: exact rows, users, cards (timed) ──
SELECT count(*) AS rows,
       count(DISTINCT user_id) AS users,
       count(DISTINCT card_oracle_id) AS cards,
       sum(shown) AS shown
FROM user_card_signal;

\echo
\echo ── 7. deck_cards: mainboard rows per card, the cost of a live "in N decks" read (timed) ──
WITH per_card AS (
    SELECT oracle_id, count(DISTINCT deck_id) AS decks,
           count(DISTINCT deck_id) FILTER (WHERE mvp_at IS NOT NULL AND mvp_at < now() - INTERVAL '3 days') AS mvp_decks
    FROM deck_cards
    WHERE board = 'deck'
    GROUP BY oracle_id
)
SELECT count(*) AS cards,
       sum(decks) AS card_deck_pairs,
       percentile_disc(0.5) WITHIN GROUP (ORDER BY decks) AS p50_decks,
       percentile_disc(0.99) WITHIN GROUP (ORDER BY decks) AS p99_decks,
       max(decks) AS max_decks,
       sum(mvp_decks) AS vested_mvps
FROM per_card;

\echo
\echo ── 8. The 10 cards in the most decks ──
SELECT lc.name, d.decks
FROM (
    SELECT oracle_id, count(DISTINCT deck_id) AS decks
    FROM deck_cards
    WHERE board = 'deck'
    GROUP BY oracle_id
    ORDER BY count(DISTINCT deck_id) DESC
    LIMIT 10
) d
LEFT JOIN latest_cards lc ON lc.oracle_id = d.oracle_id
ORDER BY d.decks DESC;

\timing off

\echo
\echo ── 9. Decks with no commander, whose swipes no per-card table keeps today, by format ──
SELECT COALESCE(format, '(none)') AS format,
       count(*) AS decks,
       count(*) FILTER (WHERE commander_id IS NULL) AS without_commander
FROM decks
GROUP BY format
ORDER BY count(*) DESC;

\echo
\echo ── 10. Writes since the stats reset, and per day ──
WITH since AS (
    SELECT GREATEST(EXTRACT(EPOCH FROM now() - COALESCE(stats_reset, pg_postmaster_start_time())) / 86400.0, 1.0 / 24) AS days
    FROM pg_stat_database
    WHERE datname = current_database()
)
SELECT s.relname AS relation,
       round(since.days::NUMERIC, 1) AS days_counted,
       s.n_tup_ins AS inserts,
       s.n_tup_upd AS updates,
       s.n_tup_hot_upd AS hot_updates,
       round((s.n_tup_ins / since.days)::NUMERIC) AS inserts_per_day,
       round((s.n_tup_upd / since.days)::NUMERIC) AS updates_per_day,
       s.n_live_tup AS live, s.n_dead_tup AS dead,
       s.last_autovacuum
FROM pg_stat_user_tables s, since
WHERE s.relname IN ('commander_card_signal', 'user_card_signal', 'commander_select_signal',
                    'otag_context_signal', 'user_week_signal', 'usage_signal_budget', 'deck_cards')
ORDER BY s.n_tup_upd + s.n_tup_ins DESC;

\echo
\echo ── 11. Signal rows last touched per day, the last 14 days (a row counts on its latest day only) ──
SELECT d.day,
       (SELECT count(*) FROM commander_card_signal WHERE updated_at::DATE = d.day) AS commander_card_rows,
       (SELECT count(*) FROM user_card_signal WHERE updated_at::DATE = d.day) AS user_card_rows,
       (SELECT COALESCE(sum(shown), 0) FROM usage_signal_budget WHERE day = d.day) AS pooled_impressions
FROM generate_series(CURRENT_DATE - 13, CURRENT_DATE, INTERVAL '1 day') AS g(ts)
CROSS JOIN LATERAL (SELECT g.ts::DATE AS day) d
ORDER BY d.day DESC;

\echo
\echo ── 12. Weekly swipes from every deck (user_week_signal), against the per-card table, the last 8 weeks ──
SELECT week_start,
       sum(swipes_right + swipes_left + swipes_up) AS swipes_all_decks,
       sum(added + skipped + maybed) AS signal_swipes,
       sum(removed) AS removals
FROM user_week_signal
WHERE week_start >= CURRENT_DATE - 56
GROUP BY week_start
ORDER BY week_start DESC;

\echo
\echo ── 13. Estimates for the two new structures ──
-- A btree entry is about 8 bytes of header plus the key plus a 4-byte line pointer, at the
-- default 90% fill. A card_swipe_signal row is a 24-byte header, a uuid, five bigints and a
-- timestamp, about 104 bytes with its line pointer; its primary key is one uuid per row.
-- Its row count is the cards with Commander signal now; every-format signal adds to that.
WITH n AS (
    SELECT count(*) AS ccs_rows,
           count(DISTINCT card_oracle_id) AS cards
    FROM commander_card_signal
)
SELECT n.cards AS card_swipe_signal_rows_now,
       pg_size_pretty((n.cards * 104 / 0.9)::BIGINT) AS card_swipe_signal_heap_est,
       pg_size_pretty((n.cards * 28 / 0.9)::BIGINT) AS card_swipe_signal_pkey_est,
       n.ccs_rows AS commander_card_signal_rows,
       pg_size_pretty((n.ccs_rows * 28 / 0.9)::BIGINT) AS card_index_on_ccs_est,
       pg_size_pretty(pg_relation_size('commander_card_signal_pkey'::regclass)) AS ccs_pkey_actual_two_uuids
FROM n;

COMMIT;
