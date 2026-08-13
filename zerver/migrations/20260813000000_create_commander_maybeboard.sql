-- Commander maybeboard: per-user "maybe this commander" list, fed by the
-- up-swipe during commander Zwipe-select (the same up = maybe gesture the
-- add screen uses for a deck's maybeboard). Per-user, NOT per-deck —
-- "collecting commanders I'm interested in" is a different intent from
-- "picking THE commander for this deck".
-- See context/plans/commander_maybeboard.md.

-- Keyed by oracle_id so an entry covers every printing; reads hydrate
-- through latest_cards, so entries always show the preferred printing.
-- Capped at 50 per user, enforced in the service/repo inside the insert
-- transaction (over-cap adds are rejected, not evicted — unlike
-- suppressions, every entry is a deliberate save). Cascade: account delete.
CREATE TABLE commander_maybeboard (
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    oracle_id  UUID        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, oracle_id)
);
