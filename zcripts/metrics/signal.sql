-- Zwipe suggestion signal — is the (commander, card) signal substrate filling
-- in, and what is it saying?
--
-- Reads commander_card_signal (global aggregate), user_card_signal (per-user
-- mirror), and the weekly tables. This is the substrate for suggestion-signal
-- Phase 3 (ranking) — run this to judge when there's enough data to rank with.
--
--   psql "$DATABASE_URL" -f zcripts/metrics/signal.sql
--   sudo -u postgres psql zwipe -f zcripts/metrics/signal.sql
--
-- Read-only. UTC. Card names resolved via scryfall_data by oracle_id.

\pset border 2
\pset null '·'
\timing off
SET TIME ZONE 'UTC';

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  ZWIPE SUGGESTION SIGNAL
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 1. SUBSTRATE SIZE — how much signal exists ──
-- Pairs = distinct (commander, card) rows. shown is the impression
-- denominator (client-derived as added + skipped + maybed).
SELECT
    (SELECT count(*) FROM commander_card_signal)                       AS global_pairs,
    (SELECT count(DISTINCT commander_oracle_id)
       FROM commander_card_signal)                                     AS commanders,
    (SELECT COALESCE(sum(shown), 0) FROM commander_card_signal)        AS shown,
    (SELECT COALESCE(sum(added), 0) FROM commander_card_signal)        AS added,
    (SELECT COALESCE(sum(skipped), 0) FROM commander_card_signal)      AS skipped,
    (SELECT COALESCE(sum(maybed), 0) FROM commander_card_signal)       AS maybed,
    (SELECT COALESCE(sum(removed), 0) FROM commander_card_signal)      AS removed,
    (SELECT count(*) FROM user_card_signal)                            AS per_user_pairs,
    (SELECT count(DISTINCT user_id) FROM user_card_signal)             AS users_contributing;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 2. COMMANDERS WITH THE MOST SIGNAL (top 15) ──
-- Where ranking would have data first. add_rate = added / shown.
SELECT
    (SELECT min(name) FROM scryfall_data sd
      WHERE sd.oracle_id = s.commander_oracle_id)          AS commander,
    sum(s.shown)                                           AS shown,
    sum(s.added)                                           AS added,
    sum(s.skipped)                                         AS skipped,
    round(100.0 * sum(s.added) / NULLIF(sum(s.shown), 0), 1) AS add_rate_pct
FROM commander_card_signal s
GROUP BY s.commander_oracle_id
ORDER BY shown DESC
LIMIT 15;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 3. STRONGEST KEEPS — pairs people consistently add (top 20) ──
-- Minimum 5 impressions so one lucky add doesn't top the list.
SELECT
    (SELECT min(name) FROM scryfall_data sd
      WHERE sd.oracle_id = s.commander_oracle_id)          AS commander,
    (SELECT min(name) FROM scryfall_data sd
      WHERE sd.oracle_id = s.card_oracle_id)               AS card,
    s.shown, s.added, s.skipped, s.removed,
    round(100.0 * s.added / NULLIF(s.shown, 0), 1)         AS add_rate_pct
FROM commander_card_signal s
WHERE s.shown >= 5
ORDER BY (s.added::numeric / NULLIF(s.shown, 0)) DESC, s.shown DESC
LIMIT 20;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 4. STRONGEST PASSES — pairs people consistently skip (top 20) ──
-- The negative signal: high-impression pairs that almost never get added.
-- removed > 0 is the strongest "doesn't fit" (added then deliberately cut).
SELECT
    (SELECT min(name) FROM scryfall_data sd
      WHERE sd.oracle_id = s.commander_oracle_id)          AS commander,
    (SELECT min(name) FROM scryfall_data sd
      WHERE sd.oracle_id = s.card_oracle_id)               AS card,
    s.shown, s.added, s.skipped, s.removed,
    round(100.0 * s.skipped / NULLIF(s.shown, 0), 1)       AS skip_rate_pct
FROM commander_card_signal s
WHERE s.shown >= 5
ORDER BY (s.skipped::numeric / NULLIF(s.shown, 0)) DESC, s.shown DESC
LIMIT 20;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 5. WEEKLY SIGNAL — per-week volume (last 8 weeks) ──
-- user_week_signal accrues from ingest (badge substrate). Weeks start
-- Monday UTC.
SELECT
    week_start,
    count(DISTINCT user_id)                                   AS users,
    sum(swipes_right + swipes_left + swipes_up + swipes_down) AS swipes,
    sum(searches)                                             AS searches,
    sum(added)                                                AS added,
    sum(skipped)                                              AS skipped,
    sum(maybed)                                               AS maybed,
    sum(removed)                                              AS removed
FROM user_week_signal
WHERE week_start >= CURRENT_DATE - INTERVAL '8 weeks'
GROUP BY week_start
ORDER BY week_start;


-- ───────────────────────────────────────────────────────────────────
\echo
\echo ── 6. FACETS — what kinds of cards get added (last 4 weeks) ──
-- Adds by mechanical category and by color identity.
SELECT facet, key, sum(added) AS added, count(DISTINCT user_id) AS users
FROM user_week_facet_signal
WHERE week_start >= CURRENT_DATE - INTERVAL '4 weeks'
GROUP BY facet, key
ORDER BY facet, added DESC;

\echo
\echo ════════════════════════════════════════════════════════════════
\echo  END OF SIGNAL REPORT
\echo ════════════════════════════════════════════════════════════════
