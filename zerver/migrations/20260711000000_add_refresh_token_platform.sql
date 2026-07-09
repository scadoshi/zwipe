-- Record the client platform (ios/android/desktop/web) on each session so we can
-- query a user's platform(s) for analytics and targeted comms. Nullable: old
-- rows and older clients stay NULL. Additive; deploy server-first.

ALTER TABLE refresh_tokens ADD COLUMN platform TEXT;

CREATE INDEX idx_refresh_tokens_platform ON refresh_tokens(platform);
