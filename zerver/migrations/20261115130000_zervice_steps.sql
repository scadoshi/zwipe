-- One row per zervice step per run (zwipe-context ideas/sync_remove_deleted_printings.md):
-- every step records when it ran, how long it took, how it ended and what it
-- counted, not only the card sync's zervice_metrics row. run_id ties a night's
-- steps together. dry_run marks a preview run, whose steps changed nothing.
-- zcripts/metrics/zervice.sql reads it.
--
-- Additive only: a new table nothing references, so the running zervice and
-- zerver keep working unchanged until their new binaries ship.
CREATE TABLE IF NOT EXISTS zervice_steps (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id      UUID        NOT NULL,
    step        TEXT        NOT NULL,
    started_at  TIMESTAMPTZ NOT NULL,
    duration_ms BIGINT      NOT NULL,
    status      TEXT        NOT NULL CHECK (status IN ('ok', 'failed', 'skipped')),
    rows        BIGINT,
    counts      JSONB       NOT NULL DEFAULT '{}'::jsonb,
    error       TEXT,
    dry_run     BOOLEAN     NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_zervice_steps_started_at ON zervice_steps (started_at);

-- zervice writes its own rows and reads nothing else here.
-- zcripts/server/sql/zervice_role.sql holds the same grant. A test database
-- has no such role.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'zervice') THEN
        GRANT SELECT, INSERT ON zervice_steps TO zervice;
    END IF;
END $$;
