-- One-time UI hint tracking. Keys are hint identifiers (e.g. "add_swipes");
-- a true value means that hint has been shown to this user. New hints are
-- introduced as new keys; no further schema changes needed.
ALTER TABLE users ADD COLUMN hints_shown jsonb NOT NULL DEFAULT '{}'::jsonb;
