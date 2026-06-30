-- First-party suggestion signal: which suggested cards players keep vs. skip
-- for a given commander. Pure aggregate keyed by (commander, card) oracle ids —
-- NO user_id, no per-swipe rows. Same privacy posture as user_lifetime_counters.
-- `shown` is the impression denominator (currently derived client-side as
-- added + skipped + maybed); added/skipped/maybed are right/left/up add-stack
-- swipes; `removed` is a delayed negative — the card was added then deliberately
-- removed from a deck (a stronger "doesn't fit" signal than a skip).
CREATE TABLE commander_card_signal (
    commander_oracle_id UUID        NOT NULL,
    card_oracle_id      UUID        NOT NULL,
    shown               BIGINT      NOT NULL DEFAULT 0,
    added               BIGINT      NOT NULL DEFAULT 0,
    skipped             BIGINT      NOT NULL DEFAULT 0,
    maybed              BIGINT      NOT NULL DEFAULT 0,
    removed             BIGINT      NOT NULL DEFAULT 0,
    updated_at          TIMESTAMP   NOT NULL DEFAULT NOW(),
    PRIMARY KEY (commander_oracle_id, card_oracle_id)
);

-- Ranking reads the pool for a commander, so index the lead key.
CREATE INDEX idx_commander_card_signal_commander
    ON commander_card_signal (commander_oracle_id);
