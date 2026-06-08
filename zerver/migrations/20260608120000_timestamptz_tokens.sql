-- Convert token-table timestamps from TIMESTAMP (wall-clock, no zone) to
-- TIMESTAMPTZ (a UTC instant). The USING ... AT TIME ZONE 'UTC' clause reads
-- the existing value as already-UTC and re-tags it. Safe because the cluster
-- has defaulted to UTC and the SQLx pool pins SET TIME ZONE 'UTC' on every
-- connection. Tokens are small, low-volume tables — safe warmup for the set.

ALTER TABLE refresh_tokens
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN expires_at TYPE TIMESTAMPTZ USING expires_at AT TIME ZONE 'UTC';

ALTER TABLE email_verification_tokens
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN expires_at TYPE TIMESTAMPTZ USING expires_at AT TIME ZONE 'UTC';

ALTER TABLE password_reset_tokens
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN expires_at TYPE TIMESTAMPTZ USING expires_at AT TIME ZONE 'UTC';
