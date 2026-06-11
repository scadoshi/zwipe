-- Synergy data layer (context/plans/synergy-data-layer.md).
--
-- Fill queue. Written and drained entirely by the synergy worker; zerver
-- never touches it. The canonical schema lives here because this repo owns
-- migrations and the read-side consumers.
CREATE TABLE synergy_requests (
    oracle_id       uuid PRIMARY KEY,
    commander_name  text NOT NULL,
    requested_at    timestamptz NOT NULL DEFAULT now(),
    attempts        int NOT NULL DEFAULT 0,
    last_attempt_at timestamptz,
    last_error      text
);

-- Cache: one row per commander, reduced jsonb payload. zerver READS this.
CREATE TABLE commander_synergy (
    oracle_id       uuid PRIMARY KEY,
    commander_name  text NOT NULL,
    payload         jsonb NOT NULL,
    fetched_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_commander_synergy_fetched_at ON commander_synergy (fetched_at);
