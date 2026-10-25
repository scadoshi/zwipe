-- Commander tag profiles for suggested commanders
-- (context/plans/suggested_commanders.md).
--
-- One row per commander with a cached synergy payload: an otag -> weight map
-- built from the payload's cards and the commander itself, weights summing
-- to 1, rare tags weighted above common ones. Written by the synergy worker
-- whenever it writes or refreshes that commander's payload; zerver reads it
-- to score commanders against a user's decks. A row older than its payload
-- (computed_at < commander_synergy.fetched_at) is stale and gets rebuilt.
--
-- The worker's scoped role needs grants on this table: re-run
-- zcripts/synergy-worker/setup_role.sh after this migration lands.

CREATE TABLE commander_tag_profiles (
    oracle_id   uuid        PRIMARY KEY,
    profile     jsonb       NOT NULL,
    computed_at timestamptz NOT NULL DEFAULT now()
);
