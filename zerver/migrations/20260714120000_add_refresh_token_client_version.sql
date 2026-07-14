-- Record the client app version (e.g. "1.6.1") on each session so we can query
-- the live version distribution for analytics and bug triage. Nullable: old
-- rows and older clients stay NULL. Additive; deploy server-first.

ALTER TABLE refresh_tokens ADD COLUMN client_version TEXT;

CREATE INDEX idx_refresh_tokens_client_version ON refresh_tokens(client_version);
