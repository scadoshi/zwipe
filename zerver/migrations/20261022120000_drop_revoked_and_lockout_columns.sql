-- refresh_tokens.revoked was replaced by hard delete and token families
-- (20261009130000), and the login lockout moved in memory and per address
-- (2026-10-10 security audit). Nothing reads these columns any more; the
-- binary that stopped selecting them deployed one release before this.

DROP INDEX IF EXISTS idx_refresh_tokens_revoked;
ALTER TABLE refresh_tokens DROP COLUMN revoked;

ALTER TABLE users
    DROP COLUMN failed_login_attempts,
    DROP COLUMN last_failed_at,
    DROP COLUMN lockout_until;
