-- Swipe memory: durable per-deck card suppressions (user feature) plus
-- per-user and per-week suggestion signal (analytics substrate — no consumer
-- yet). Clearing suppressions deletes deck_card_suppressions rows ONLY;
-- signal tables are never touched by the user-facing clear.
-- See context/plans/swipe_memory.md.

-- Suppression set: cards the deck-aware search must not serve this deck,
-- keyed by oracle_id so a suppression covers every printing. `source` is
-- provenance, not identity — it stays out of the PK: a card is suppressed
-- for a deck or it isn't (one row), and we know why and when. Sources:
-- 'skip' (Add-screen left swipe) and 'removal' (deliberate single-card
-- removal). No default and CHECK-enforced: every insert must state its
-- provenance; a new source requires a migration. Capped at 5,000 per deck
-- at ingest (evict oldest). Cascade: deck delete / account delete (via decks).
CREATE TABLE deck_card_suppressions (
    deck_id       UUID        NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    oracle_id     UUID        NOT NULL,
    source        TEXT        NOT NULL CHECK (source IN ('skip', 'removal')),
    suppressed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (deck_id, oracle_id)
);

-- Per-user suggestion signal: mirrors commander_card_signal with user_id as
-- the leading key. Deletion correctness via FK cascade.
CREATE TABLE user_card_signal (
    user_id             UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    commander_oracle_id UUID        NOT NULL,
    card_oracle_id      UUID        NOT NULL,
    shown               BIGINT      NOT NULL DEFAULT 0,
    added               BIGINT      NOT NULL DEFAULT 0,
    skipped             BIGINT      NOT NULL DEFAULT 0,
    maybed              BIGINT      NOT NULL DEFAULT 0,
    removed             BIGINT      NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, commander_oracle_id, card_oracle_id)
);

-- Weekly scalar counters (ISO week, Monday UTC). Badge substrate; no
-- consumer yet — accrues from ingest so weekly history exists from day one.
CREATE TABLE user_week_signal (
    user_id      UUID    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start   DATE    NOT NULL,
    swipes_right INTEGER NOT NULL DEFAULT 0,
    swipes_left  INTEGER NOT NULL DEFAULT 0,
    swipes_up    INTEGER NOT NULL DEFAULT 0,
    swipes_down  INTEGER NOT NULL DEFAULT 0,
    searches     INTEGER NOT NULL DEFAULT 0,
    added        INTEGER NOT NULL DEFAULT 0,
    skipped      INTEGER NOT NULL DEFAULT 0,
    maybed       INTEGER NOT NULL DEFAULT 0,
    removed      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, week_start)
);

-- Weekly facet counters (accepts by mechanical category / color identity).
-- Normalized rows, not jsonb: adding a counter stays a plain upsert.
CREATE TABLE user_week_facet_signal (
    user_id    UUID    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start DATE    NOT NULL,
    facet      TEXT    NOT NULL,  -- 'category' | 'color'
    key        TEXT    NOT NULL,  -- e.g. 'Removal' / 'W'
    added      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, week_start, facet, key)
);
