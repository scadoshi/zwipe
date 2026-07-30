-- Client error + crash reporting (context/plans/client-error-reporting.md).
--
-- Both tables are aggregate debugging signal: NO user identity, same privacy
-- posture as the signal tables. client_version/platform are self-reported by
-- the client (untrusted metadata, truncated server-side before insert).
--
-- Retention: pruned to ~90 days by zervice's upkeep step. Grants for the
-- scoped zervice role live in zcripts/server/sql/zervice_role.sql (roles are
-- per-cluster infrastructure, not schema — this migration must run on dev/test
-- databases that have no zervice role).

-- One row per handled-error report entry (client dedupes within a flush
-- window; `count` carries the repeats).
CREATE TABLE client_errors (
    id             BIGSERIAL   PRIMARY KEY,
    received_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    client_version TEXT        NOT NULL,
    platform       TEXT        NOT NULL,
    screen         TEXT        NOT NULL,
    component      TEXT        NOT NULL DEFAULT '',
    action         TEXT        NOT NULL,
    kind           TEXT        NOT NULL,
    message        TEXT        NOT NULL,
    count          INTEGER     NOT NULL DEFAULT 1
);

-- Reading pattern: errors by (client_version, screen, kind) per day; the
-- upkeep prune deletes on received_at.
CREATE INDEX idx_client_errors_received_at ON client_errors (received_at);

-- One row per crash, exactly-once via the client-stamped crash_id
-- (`INSERT ... ON CONFLICT (crash_id) DO NOTHING`); the client deletes its
-- crash file only on 2xx and retries next launch otherwise.
CREATE TABLE crash_reports (
    crash_id       UUID        PRIMARY KEY,
    received_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    occurred_at    TIMESTAMPTZ NOT NULL,
    client_version TEXT        NOT NULL,
    platform       TEXT        NOT NULL,
    message        TEXT        NOT NULL
);

CREATE INDEX idx_crash_reports_received_at ON crash_reports (received_at);
