-- Phase 5 (otags): generalized-context per-otag swipe signal — the cross-format
-- "moat" dataset. It accrues BEFORE anything serves on it (see
-- context/plans/otags/moat.md + sequencing.md Phase 5).
--
-- One row per (generalized context, oracle tag). For every add-stack swipe the
-- server credits each OTAG OF THE SWIPED CARD (from card_profiles.oracle_tags),
-- keyed by the deck's generalized context:
--   * Commander decks: context_key = 'commander:<commander_oracle_id>'
--       (works for every existing client — they already send the commander).
--   * Non-Commander decks: context_key = 'format_ci:<format>:<color_identity>'
--       (format = Format::as_str; color identity = canonical WUBRG codes with
--        colorless -> 'C'), derived server-side from the deck via deck_id.
-- Nothing about otags or format/CI is on the wire — it is all derived server-side.
--
-- Pure aggregate — NO user_id, no per-swipe rows (same privacy posture as
-- commander_card_signal). context_key is a single TEXT so a commander-OR-(format,
-- CI) union can live in the PRIMARY KEY (nullable PK columns aren't allowed); a
-- prefix keeps future context kinds (e.g. 'deck:<id>') additive.
--
-- shown is the impression denominator; added/skipped/maybed are right/left/up
-- add-stack swipes; removed is a delayed negative (added then deliberately cut).
CREATE TABLE otag_context_signal (
    context_key TEXT        NOT NULL,
    oracle_tag  TEXT        NOT NULL,
    shown       BIGINT      NOT NULL DEFAULT 0,
    added       BIGINT      NOT NULL DEFAULT 0,
    skipped     BIGINT      NOT NULL DEFAULT 0,
    maybed      BIGINT      NOT NULL DEFAULT 0,
    removed     BIGINT      NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (context_key, oracle_tag)
);

-- Serving (Phase 6) reads a whole context at once, so index the lead key.
CREATE INDEX idx_otag_context_signal_context ON otag_context_signal (context_key);

-- Nightly rollup (refreshed by zervice, mirroring card_signal_rollup): precomputes
-- the net/shown floats a serve term reads. net encodes what each gesture means —
-- an add is full credit, a maybe half (expressed interest; impression-only would
-- make it arithmetically identical to a skip), a removal a full take-back. Skips
-- need no term: shown = added + skipped + maybed, so every skip is denominator
-- drag. net can go negative — that's demotion.
CREATE MATERIALIZED VIEW otag_context_signal_rollup AS
SELECT context_key,
       oracle_tag,
       SUM(added + 0.5 * maybed - removed)::float8 AS net,
       SUM(shown)::float8                          AS shown
FROM otag_context_signal
GROUP BY context_key, oracle_tag;

CREATE UNIQUE INDEX idx_otag_context_signal_rollup
    ON otag_context_signal_rollup (context_key, oracle_tag);
