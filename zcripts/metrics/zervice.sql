-- zervice: what the nightly job does to the database.
--
-- Run it:
--   psql "$DATABASE_URL" -f zcripts/metrics/zervice.sql
-- or on the server:
--   sudo -u postgres psql zwipe -f zcripts/metrics/zervice.sql
--
-- zervice (zerver/src/bin/zervice.rs) runs five steps in order, each non-fatal:
--   1. card sync: Scryfall default_cards into scryfall_data, card_profiles and oou_sets,
--      then (1b) the removal of the printings Scryfall deleted, which only previews until
--      PRINTING_REMOVAL=on
--   2. oracle tags: oracle_tags and card_oracle_tags, then the tag projection and grouping
--   3. derive categories: card_profiles roles from oracle tags, with straggler merges
--   4. refresh materialized views: latest_cards, card_signal_rollup, otag_context_signal_rollup
--   5. upkeep prunes: client_errors and crash_reports past 90 days, refresh_tokens expired
--      or replaced more than 7 days ago, expired email_change_requests
--
-- Every step records a zervice_steps row per run (step, start, duration, status, its main
-- count and its named counts, the error when it failed, and whether the run was a dry
-- run): sections 4b to 4e read them. The card sync also keeps its zervice_metrics row
-- (rows received, upserted, skipped, failed, and each failed card), which a dry run does
-- not write. Before zervice_steps existed, steps 2 to 5 recorded nothing, so sections 5
-- to 9 still read the tables they leave behind. Section 10 lists the gaps.
--
-- Read-only: no writes, no locks.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZERVICE: THE NIGHTLY JOB
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. LAST 14 RUNS of the card sync (step 1) ──
SELECT
    to_char(started_at, 'YYYY-MM-DD HH24:MI')          AS started,
    to_char(ended_at, 'HH24:MI')                         AS ended,
    duration_in_seconds                                  AS secs,
    status,
    received_count                                       AS received,
    upserted_count                                       AS upserted,
    skipped_count                                        AS skipped,
    error_count                                          AS failed,
    received_count - upserted_count - skipped_count      AS unaccounted
FROM zervice_metrics
ORDER BY started_at DESC
LIMIT 14;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. HEALTH — is it running every night? ──
-- A run every day is the expectation; days_without_run counts calendar days in the last 14
-- with no run started. A run left in_progress means zervice died mid-sync.
SELECT
    (SELECT count(*) FROM zervice_metrics)                                          AS runs_all_time,
    (SELECT min(started_at)::date FROM zervice_metrics)                             AS first_run,
    (SELECT max(started_at) FROM zervice_metrics)                                   AS last_run_at,
    round(extract(epoch FROM now() - (SELECT max(started_at) FROM zervice_metrics)) / 3600, 1)
                                                                                    AS hours_since_last,
    (SELECT count(*) FROM generate_series(current_date - 13, current_date, '1 day') d
       WHERE NOT EXISTS (SELECT 1 FROM zervice_metrics m WHERE m.started_at::date = d::date))
                                                                                    AS days_without_run_14d,
    (SELECT count(*) FROM zervice_metrics WHERE status = 'in_progress')             AS stuck_in_progress,
    (SELECT count(*) FROM zervice_metrics WHERE status <> 'success'
       AND started_at > now() - interval '30 days')                                 AS non_success_30d;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. TRENDS by week ──
-- Duration and volume per Monday-UTC week. A jump in received means Scryfall's bulk file
-- grew (a new set); a jump in skipped or failed is the thing to chase.
SELECT
    date_trunc('week', started_at)::date        AS week,
    count(*)                                    AS runs,
    round(avg(duration_in_seconds))             AS avg_secs,
    max(duration_in_seconds)                    AS max_secs,
    round(avg(received_count))                  AS avg_received,
    round(avg(upserted_count))                  AS avg_upserted,
    round(avg(skipped_count))                   AS avg_skipped,
    sum(error_count)                            AS failed,
    string_agg(DISTINCT status, ', ')           AS statuses
FROM zervice_metrics
GROUP BY 1
ORDER BY 1 DESC
LIMIT 12;

\echo
\echo ── 3b. Night over night: what moved since the run before ──
SELECT
    to_char(started_at, 'YYYY-MM-DD HH24:MI')                                 AS run,
    received_count - lag(received_count) OVER (ORDER BY started_at)           AS received_delta,
    upserted_count - lag(upserted_count) OVER (ORDER BY started_at)           AS upserted_delta,
    duration_in_seconds - lag(duration_in_seconds) OVER (ORDER BY started_at) AS secs_delta
FROM zervice_metrics
ORDER BY started_at DESC
LIMIT 14;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. FAILED CARDS in the last 14 runs ──
-- Each run keeps the cards it could not write, with the error. Empty is good.
SELECT
    to_char(m.started_at, 'YYYY-MM-DD')     AS night,
    e->>'card_name'                         AS card,
    e->>'card_id'                           AS scryfall_id,
    left(e->>'error', 120)                  AS error
FROM (SELECT * FROM zervice_metrics ORDER BY started_at DESC LIMIT 14) m,
     jsonb_array_elements(m.errors) e
ORDER BY m.started_at DESC
LIMIT 25;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4b. LAST NIGHT, step by step (zervice_steps) ──
-- The latest run in run order. status: ok, failed or skipped; rows: the step's main count
-- (printings received, rows written, rows pruned); dry: a preview whose writes rolled back.
WITH last_run AS (
    SELECT run_id FROM zervice_steps ORDER BY started_at DESC LIMIT 1
)
SELECT
    to_char(s.started_at, 'YYYY-MM-DD HH24:MI:SS')   AS started,
    s.step,
    s.status,
    round(s.duration_ms / 1000.0, 1)                 AS secs,
    s.rows,
    s.counts,
    s.dry_run                                        AS dry,
    left(s.error, 100)                               AS error
FROM zervice_steps s
JOIN last_run USING (run_id)
ORDER BY s.started_at;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4c. EACH STEP over the last 14 real nights ──
-- Real runs only (dry runs left out). A step whose ok count trails the others, or whose
-- duration climbs, is the one to chase.
SELECT
    step,
    count(*)                                             AS runs,
    count(*) FILTER (WHERE status = 'ok')                AS ok,
    count(*) FILTER (WHERE status = 'failed')            AS failed,
    count(*) FILTER (WHERE status = 'skipped')           AS skipped,
    round(avg(duration_ms) / 1000.0, 1)                  AS avg_secs,
    round(max(duration_ms) / 1000.0, 1)                  AS max_secs,
    round(avg(rows))                                     AS avg_rows,
    max(started_at)::date                                AS last_run
FROM zervice_steps
WHERE NOT dry_run AND started_at > now() - interval '14 days'
GROUP BY step
ORDER BY min(started_at::time), step;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4d. FAILED OR SKIPPED steps in the last 14 days ──
-- Empty is good. A skipped printing_removal means the card sync did not complete; skipped
-- refreshes mean a dry run.
SELECT
    to_char(started_at, 'YYYY-MM-DD HH24:MI')    AS started,
    step,
    status,
    dry_run                                      AS dry,
    coalesce(left(error, 120), counts->>'skipped') AS why
FROM zervice_steps
WHERE status <> 'ok' AND started_at > now() - interval '14 days'
ORDER BY started_at DESC
LIMIT 30;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4e. PRINTING REMOVAL: what each night found and did ──
-- to_remove and kept are the plan; removed and the moves are what an applied run did
-- (applied = 1). Until PRINTING_REMOVAL=on, every row is a preview with removed 0.
SELECT
    to_char(started_at, 'YYYY-MM-DD')            AS night,
    status,
    dry_run                                      AS dry,
    (counts->>'applied')::int                    AS applied,
    (counts->>'to_remove')::int                  AS to_remove,
    (counts->>'kept')::int                       AS kept,
    (counts->>'removed')::int                    AS removed,
    (counts->>'moved_deck_cards')::int           AS moved_cards,
    (counts->>'moved_slots')::int                AS moved_slots
FROM zervice_steps
WHERE step = 'printing_removal'
ORDER BY started_at DESC
LIMIT 14;

-- Every printing removed so far, newest first, with where its references went.
SELECT
    to_char(rp.removed_at, 'YYYY-MM-DD')         AS removed,
    rp.id                                        AS printing,
    rp.replacement_id                            AS replaced_by,
    sd.name                                      AS replacement_name,
    sd.set                                       AS replacement_set
FROM removed_printings rp
LEFT JOIN scryfall_data sd ON sd.id = rp.replacement_id
ORDER BY rp.removed_at DESC
LIMIT 25;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. CARD CATALOG (what step 1 maintains) ──
SELECT
    (SELECT count(*) FROM scryfall_data)                        AS printings,
    (SELECT count(DISTINCT oracle_id) FROM scryfall_data)       AS cards,
    round((SELECT count(*) FROM scryfall_data)::numeric
          / NULLIF((SELECT count(DISTINCT oracle_id) FROM scryfall_data), 0), 2)
                                                                AS printings_per_card,
    (SELECT count(*) FROM card_profiles)                        AS card_profiles,
    (SELECT count(*) FROM oou_sets)                             AS oou_sets,
    pg_size_pretty(pg_total_relation_size('scryfall_data'))     AS scryfall_data_size;

\echo
\echo ── 5b. Printings per card ──
SELECT
    percentile_cont(0.5) WITHIN GROUP (ORDER BY n)   AS median,
    percentile_cont(0.9) WITHIN GROUP (ORDER BY n)   AS p90,
    max(n)                                           AS max,
    count(*) FILTER (WHERE n = 1)                    AS cards_with_one_printing
FROM (SELECT count(*) AS n FROM scryfall_data GROUP BY oracle_id) per_card;

\echo
\echo ── 5c. Newest sets in the catalog ──
SELECT set, set_name, max(released_at) AS released, count(*) AS printings
FROM scryfall_data
GROUP BY set, set_name
ORDER BY released DESC NULLS LAST
LIMIT 6;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 6. ORACLE TAGS AND CATEGORIES (steps 2 and 3, not recorded) ──
-- Read from what the steps leave behind: when a tag or a profile was last touched.
SELECT
    (SELECT count(*) FROM oracle_tags)                                            AS oracle_tags,
    (SELECT max(updated_at) FROM oracle_tags)                                     AS tags_last_updated,
    (SELECT count(*) FROM oracle_tags WHERE updated_at > now() - interval '1 day') AS tags_updated_24h,
    (SELECT count(*) FROM card_oracle_tags)                                       AS card_tag_links,
    (SELECT count(DISTINCT oracle_id) FROM card_oracle_tags)                      AS cards_tagged,
    (SELECT max(updated_at) FROM card_profiles)                                   AS profiles_last_updated,
    (SELECT count(*) FROM card_profiles WHERE updated_at > now() - interval '1 day')
                                                                                  AS profiles_updated_24h;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 7. MATERIALIZED VIEWS (step 4, refresh timing not recorded) ──
-- Postgres keeps no refresh time. A refresh rewrites the view, so autoanalyze usually
-- follows it within minutes: last_autoanalyze is a rough stand-in, not a record. It means
-- something for latest_cards; the rollups are small enough that autoanalyze often skips them.
SELECT
    c.relname                                       AS view,
    s.n_live_tup                                    AS rows_estimate,
    pg_size_pretty(pg_total_relation_size(c.oid))   AS size,
    greatest(s.last_analyze, s.last_autoanalyze)    AS last_analyzed_proxy
FROM pg_class c
LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
WHERE c.relkind = 'm'
ORDER BY c.relname;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 8. RETENTION PRUNES (step 5, counts not recorded) ──
-- After a good night, past_retention should be 0: anything there survived a prune.
-- next_7d is what the coming week's prunes will remove.
SELECT
    'client_errors'                                                               AS tbl,
    count(*)                                                                      AS rows,
    min(received_at)                                                              AS oldest,
    count(*) FILTER (WHERE received_at < now() - interval '90 days')              AS past_retention,
    count(*) FILTER (WHERE received_at < now() - interval '83 days'
                       AND received_at >= now() - interval '90 days')             AS next_7d
FROM client_errors
UNION ALL
SELECT
    'crash_reports',
    count(*),
    min(received_at),
    count(*) FILTER (WHERE received_at < now() - interval '90 days'),
    count(*) FILTER (WHERE received_at < now() - interval '83 days'
                       AND received_at >= now() - interval '90 days')
FROM crash_reports;

\echo
\echo ── 8b. Sessions and email changes ──
-- prunable_now matches the upkeep delete: never replaced and expired, or replaced more than
-- 7 days ago. Non-zero after a run means the prune did not run or failed.
SELECT
    (SELECT count(*) FROM refresh_tokens)                                          AS refresh_tokens,
    (SELECT count(*) FROM refresh_tokens WHERE replaced_at IS NULL AND expires_at >= now())
                                                                                   AS live,
    (SELECT count(*) FROM refresh_tokens WHERE replaced_at >= now() - interval '7 days')
                                                                                   AS replaced_kept_7d,
    (SELECT count(*) FROM refresh_tokens
       WHERE (replaced_at IS NULL AND expires_at < now())
          OR replaced_at < now() - interval '7 days')                              AS prunable_now,
    (SELECT count(*) FROM email_change_requests)                                   AS email_changes,
    (SELECT count(*) FROM email_change_requests WHERE expires_at < now())          AS email_changes_expired;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 9. SUPPRESSIONS CAP (not a zervice step) ──
-- The 5,000-per-deck cap on durable skips is enforced when a skip is written, not at night;
-- this shows how close any deck is to it.
SELECT
    count(*)                                        AS suppressions,
    count(DISTINCT deck_id)                         AS decks,
    max(per_deck)                                   AS most_in_one_deck,
    count(DISTINCT deck_id) FILTER (WHERE per_deck >= 4500)
                                                    AS decks_near_cap
FROM (SELECT deck_id, count(*) OVER (PARTITION BY deck_id) AS per_deck
      FROM deck_card_suppressions) s;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 10. GAPS — what zervice does that leaves no record ──
\echo   Every step now records a zervice_steps row (sections 4b to 4e), so the counts steps 2
\echo   to 5 used to keep only in the log files are in the database from the first night after
\echo   the deploy; nights before that have no step rows.
\echo   Still open: step 1 records no count of printings that actually changed (zervice_metrics
\echo   has upserted, not changed, and scryfall_data has no updated_at), and no per-table split
\echo   between scryfall_data, card_profiles and oou_sets. A view refresh records its duration
\echo   but no row count, since Postgres reports none.
\echo
