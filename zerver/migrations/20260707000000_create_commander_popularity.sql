-- Decks-helmed popularity per commander (context/plans/commander_select_ordering.md).
--
-- Written by the synergy worker (zynergy → context/plans/commander-popularity.md),
-- read by zerver's commander-select ordering. The canonical schema lives here
-- because this repo owns migrations and the read-side consumer — same split as
-- the commander_synergy tables.
--
-- Contract zerver assumes: full-pool coverage is NOT guaranteed (absent rows are
-- normal — the select ORDER BY falls back to edhrec_rank in
-- outbound/sqlx/card/mod.rs); `decks` is a comparable magnitude within the table;
-- rows refresh on the worker's cadence (weeks-stale is fine). Empty table = pure
-- edhrec_rank fallback, which is the revert lever (TRUNCATE to switch off).
CREATE TABLE commander_popularity (
    oracle_id  uuid PRIMARY KEY,
    name       text NOT NULL,
    decks      bigint NOT NULL,
    fetched_at timestamptz NOT NULL DEFAULT now()
);
