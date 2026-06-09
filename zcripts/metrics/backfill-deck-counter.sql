-- One-time backfill: reconcile user_lifetime_counters.decks_created to the
-- decks table, so the public "Decks created" marketing number on zwipe.net
-- stops missing decks that were built BEFORE the metrics handler deployed.
--
-- WHY this is needed:
--   The public endpoint serves SUM(user_lifetime_counters.decks_created).
--   That counter is incremented server-side on every create/clone — but only
--   since the metrics code deployed. Decks built before then never bumped it,
--   so their builders sit at decks_created = 0 while clearly owning decks.
--
-- WHY it's safe (and stays "marketing-friendly"):
--   The UPDATE only ever RAISES a counter to the user's current live deck
--   count, never lowers it (WHERE decks_created < live_count). Users whose
--   counter is already higher than their live decks (e.g. created-then-deleted)
--   are left untouched — the monotonic, never-drops property is preserved.
--
-- Run it:
--   sudo -u postgres psql zwipe -f zcripts/metrics/backfill-deck-counter.sql
--   psql "$DATABASE_URL" -f zcripts/metrics/backfill-deck-counter.sql
--
-- Wrapped in a transaction. It prints a preview and the before/after totals;
-- inspect them, and it COMMITs at the end. To abort instead, Ctrl-C before the
-- COMMIT or change the final COMMIT to ROLLBACK.

\pset border 2
SET TIME ZONE 'UTC';

BEGIN;

\echo
\echo ── BEFORE — current public total (SUM of counters) ──
SELECT COALESCE(SUM(decks_created), 0) AS public_decks_created
FROM user_lifetime_counters;

\echo
\echo ── PREVIEW — users whose counter will be raised ──
-- live = decks they currently own; counter = what the marketing number uses.
SELECT u.username,
       c.decks_created            AS counter_now,
       live.cnt                   AS live_decks,
       live.cnt - c.decks_created AS will_add
FROM user_lifetime_counters c
JOIN users u ON u.id = c.user_id
JOIN (SELECT user_id, count(*) AS cnt FROM decks GROUP BY user_id) live
     ON live.user_id = c.user_id
WHERE c.decks_created < live.cnt
ORDER BY will_add DESC;

\echo
\echo ── APPLYING ──
UPDATE user_lifetime_counters c
SET decks_created = sub.cnt
FROM (SELECT user_id, count(*) AS cnt FROM decks GROUP BY user_id) sub
WHERE c.user_id = sub.user_id
  AND c.decks_created < sub.cnt;

\echo
\echo ── AFTER — reconciled public total ──
SELECT COALESCE(SUM(decks_created), 0) AS public_decks_created
FROM user_lifetime_counters;

COMMIT;

\echo
\echo Done. The public marketing number now reflects pre-deploy decks too.
