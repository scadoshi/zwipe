-- Pre-auth funnel events: what happens before a user exists (app opened,
-- register screen viewed, register submitted). Keyed by a random
-- client-generated session UUID — no user_id, no PII, nothing to cascade
-- from. Registration success itself lives in user_events ('register'), so
-- funnel drop-off is read as distinct-session counts per kind against that.
-- Kinds are a closed enum at ingest (AnonymousEventKind in zwipe-core);
-- the column stays TEXT so a new kind is a code change, not a migration.
CREATE TABLE anonymous_events (
    id          BIGSERIAL   PRIMARY KEY,
    session_id  UUID        NOT NULL,
    kind        TEXT        NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_anonymous_events_session ON anonymous_events(session_id, occurred_at);
CREATE INDEX idx_anonymous_events_kind ON anonymous_events(kind, occurred_at DESC);
