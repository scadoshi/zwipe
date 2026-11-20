-- Oracle tags check: why would the Activity screen's Oracle tags panel stay empty
-- ("Swipe more cards to see your Oracle tags") while Card roles above it has data?
--
-- Both panels read user_card_signal (decks with a commander) through latest_cards into
-- card_profiles: Card roles reads card_profiles.card_roles, Oracle tags reads
-- card_profiles.oracle_tags_by_role, which zervice's oracle_tag_grouping step writes. A tag
-- shows once the account has seen 10 cards with it (ACTIVITY_MIN_SEEN). This prints the two
-- columns' health across the catalog, the nightly steps that write them, and, for one
-- account, how many of its swiped cards carry each column and how many tags pass the bar.
--
-- Run it, naming the account (its name is only compared, never printed):
--   psql "$DATABASE_URL" -v who=<username> -f zcripts/metrics/oracle_tags_check.sql
-- or on the server:
--   sudo -u postgres psql zwipe -v who=<username> -f zcripts/metrics/oracle_tags_check.sql
-- Without -v who, it reads the account with the most dealt cards in user_card_signal.
--
-- Read-only: one READ ONLY transaction with a statement timeout. No user id or name in the
-- output, only counts and tag slugs. Keep section 4 in step with fetch_activity_counts in
-- zerver/src/lib/outbound/sqlx/deck/mod.rs.

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
\echo  ZWIPE ORACLE TAGS CHECK
\echo ════════════════════════════════════════════════════════════════
SELECT now() AS generated_at_utc;

\echo
\echo ── 1. The two columns across the catalog ──
-- empty_* counts profiles whose column is NULL or empty. Most cards carry both; a column
-- empty on nearly every row is the one the panel lost.
SELECT
    'every printing' AS scope,
    count(*) AS profiles,
    count(*) FILTER (WHERE card_roles IS NULL OR card_roles = '[]'::jsonb) AS empty_roles,
    count(*) FILTER (WHERE oracle_tags IS NULL OR oracle_tags = '[]'::jsonb) AS empty_tags,
    count(*) FILTER (WHERE oracle_tags_by_role IS NULL OR oracle_tags_by_role = '{}'::jsonb)
        AS empty_tags_by_role,
    max(updated_at) AS last_updated
FROM card_profiles
UNION ALL
SELECT
    'latest_cards',
    count(*),
    count(*) FILTER (WHERE cp.card_roles IS NULL OR cp.card_roles = '[]'::jsonb),
    count(*) FILTER (WHERE cp.oracle_tags IS NULL OR cp.oracle_tags = '[]'::jsonb),
    count(*) FILTER (WHERE cp.oracle_tags_by_role IS NULL OR cp.oracle_tags_by_role = '{}'::jsonb),
    max(cp.updated_at)
FROM latest_cards lc
JOIN card_profiles cp ON cp.scryfall_data_id = lc.id;

SELECT
    (SELECT count(*) FROM oracle_tags) AS oracle_tags,
    (SELECT count(*) FROM oracle_tags WHERE cardinality(parent_ids) > 0) AS tags_with_parents,
    (SELECT max(updated_at) FROM oracle_tags) AS tags_last_updated,
    (SELECT count(*) FROM card_oracle_tags) AS card_tag_links,
    (SELECT count(DISTINCT oracle_id) FROM card_oracle_tags) AS cards_tagged;

\echo
\echo ── 2. The nightly steps that write them, last 7 real nights ──
-- rows: profiles each step wrote. A grouping step that failed, skipped or wrote 0 leaves
-- oracle_tags_by_role stale or empty.
SELECT
    to_char(started_at, 'YYYY-MM-DD HH24:MI') AS started,
    step,
    status,
    rows,
    counts,
    left(error, 100) AS error
FROM zervice_steps
WHERE NOT dry_run
  AND step IN ('oracle_tags', 'oracle_tag_projection', 'oracle_tag_grouping',
               'derive_categories', 'refresh_latest_cards')
  AND started_at > now() - interval '7 days'
ORDER BY started_at DESC;

-- The account: -v who=<username>, else the one with the most dealt cards.
\if :{?who}
SELECT id AS who_id FROM users WHERE username = :'who' \gset
\else
SELECT user_id AS who_id FROM user_card_signal
GROUP BY user_id ORDER BY sum(added + skipped + maybed) DESC, user_id LIMIT 1 \gset
\endif

\echo
\echo ── 3. One account: swiped cards and what they carry ──
-- signal_rows: (commander, card) rows in user_card_signal. in_latest: rows whose card is in
-- latest_cards. with_roles / with_tags_by_role: those whose latest printing carries each
-- column. seen: added + skipped + maybed, what both panels count.
SELECT
    count(*) AS signal_rows,
    sum(s.added + s.skipped + s.maybed) AS seen,
    count(lc.id) AS in_latest,
    count(*) FILTER (WHERE cp.card_roles IS NOT NULL AND cp.card_roles <> '[]'::jsonb)
        AS with_roles,
    count(*) FILTER (WHERE cp.oracle_tags_by_role IS NOT NULL
                       AND cp.oracle_tags_by_role <> '{}'::jsonb) AS with_tags_by_role,
    count(*) FILTER (WHERE cp.oracle_tags IS NOT NULL AND cp.oracle_tags <> '[]'::jsonb)
        AS with_tags,
    min(s.updated_at)::date AS oldest_write,
    max(s.updated_at)::date AS newest_write
FROM user_card_signal s
LEFT JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
LEFT JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
WHERE s.user_id = :'who_id';

\echo
\echo ── 4. One account: roles and tags against the bar of 10 ──
-- The Activity query's own grouping. roles_over_10 is what Card roles draws from;
-- tags_over_10 is what Oracle tags draws from, so 0 there is the empty panel.
WITH roles AS (
    SELECT r.role, SUM(s.added + s.skipped + s.maybed) AS seen
    FROM user_card_signal s
    JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
    JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
    CROSS JOIN LATERAL jsonb_array_elements_text(COALESCE(cp.card_roles, '[]'::jsonb)) AS r(role)
    WHERE s.user_id = :'who_id'
    GROUP BY r.role
),
tagged AS (
    SELECT s.commander_oracle_id, s.card_oracle_id, t.tag,
           MIN(s.added + s.skipped + s.maybed) AS seen
    FROM user_card_signal s
    JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
    JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
    CROSS JOIN LATERAL jsonb_each(COALESCE(cp.oracle_tags_by_role, '{}'::jsonb)) AS r(role, slugs)
    CROSS JOIN LATERAL jsonb_array_elements_text(r.slugs) AS t(tag)
    WHERE s.user_id = :'who_id' AND t.tag NOT LIKE 'cycle-%'
    GROUP BY s.commander_oracle_id, s.card_oracle_id, t.tag
),
tags AS (
    SELECT tag, SUM(seen) AS seen FROM tagged GROUP BY tag
)
SELECT
    (SELECT count(*) FROM roles) AS roles,
    (SELECT count(*) FROM roles WHERE seen >= 10) AS roles_over_10,
    (SELECT max(seen) FROM roles) AS most_seen_role,
    (SELECT count(*) FROM tags) AS tags,
    (SELECT count(*) FROM tags WHERE seen >= 10) AS tags_over_10,
    (SELECT max(seen) FROM tags) AS most_seen_tag;

\echo
\echo ── 4b. Where one account swipes went ──
-- totals_seen: what the Totals panel counts, every deck (user_week_signal). commander_seen:
-- what roles and tags count, decks with a commander only (user_card_signal). A wide gap
-- means most swipes land in decks with no commander, or past the daily signal budget.
SELECT
    (SELECT COALESCE(sum(added + skipped + maybed), 0) FROM user_week_signal
      WHERE user_id = :'who_id') AS totals_seen,
    (SELECT COALESCE(sum(added + skipped + maybed), 0) FROM user_card_signal
      WHERE user_id = :'who_id') AS commander_seen,
    (SELECT count(*) FROM decks WHERE user_id = :'who_id') AS decks,
    (SELECT count(*) FROM decks WHERE user_id = :'who_id' AND commander_id IS NOT NULL)
        AS decks_with_commander;

\echo
\echo ── 4c. Totals and commander swipes by week ──
SELECT
    w.week_start AS week,
    w.added + w.skipped + w.maybed AS totals_seen,
    COALESCE(c.seen, 0) AS commander_rows_last_written
FROM user_week_signal w
LEFT JOIN (
    SELECT date_trunc('week', updated_at)::date AS week, sum(added + skipped + maybed) AS seen
    FROM user_card_signal
    WHERE user_id = :'who_id'
    GROUP BY 1
) c ON c.week = w.week_start
WHERE w.user_id = :'who_id'
ORDER BY w.week_start DESC
LIMIT 8;

\echo
\echo ── 4d. The daily signal budget, the days still kept ──
-- Past 5,000 impressions (adds, skips and saves) in a UTC day, a flush's per-card signals
-- and its totals are dropped; only the heatmap counts it.
SELECT day, shown, shown > 5000 AS over_budget
FROM usage_signal_budget
WHERE user_id = :'who_id'
ORDER BY day DESC;

\echo
\echo ── 5. One account: most seen tags (slugs only) ──
SELECT t.tag, SUM(t.seen) AS seen
FROM (
    SELECT s.commander_oracle_id, s.card_oracle_id, tg.tag,
           MIN(s.added + s.skipped + s.maybed) AS seen
    FROM user_card_signal s
    JOIN latest_cards lc ON lc.oracle_id = s.card_oracle_id
    JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
    CROSS JOIN LATERAL jsonb_each(COALESCE(cp.oracle_tags_by_role, '{}'::jsonb)) AS r(role, slugs)
    CROSS JOIN LATERAL jsonb_array_elements_text(r.slugs) AS tg(tag)
    WHERE s.user_id = :'who_id' AND tg.tag NOT LIKE 'cycle-%'
    GROUP BY s.commander_oracle_id, s.card_oracle_id, tg.tag
) t
GROUP BY t.tag
ORDER BY seen DESC, t.tag
LIMIT 10;

ROLLBACK;
