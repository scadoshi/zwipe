-- Pile-end telemetry: one row each time a deck's add-screen pile runs out, so
-- we can see how often the pool ends and for which commanders.
--
-- Aggregate signal with NO user or deck identity, the same privacy posture as
-- client_errors. Ingest verifies the reporting user owns the deck, then keeps
-- only the deck's commander (NULL for a deck without one).

CREATE TABLE pile_end_events (
    id                  BIGSERIAL   PRIMARY KEY,
    received_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    kind                TEXT        NOT NULL CHECK (kind IN ('empty', 'synergy_exhausted')),
    synergy             BOOLEAN     NOT NULL,
    commander_oracle_id UUID
);

-- Reading pattern: pile ends per day, by kind and commander.
CREATE INDEX idx_pile_end_events_received_at ON pile_end_events (received_at);
