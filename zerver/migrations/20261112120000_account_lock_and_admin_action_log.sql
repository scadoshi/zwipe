-- An account an admin locks from the server: sign-in and session refresh refuse it until unlocked.
-- Both columns are NULL for every existing account, which keeps it signing in as before.
ALTER TABLE users
    ADD COLUMN locked_at TIMESTAMPTZ,
    ADD COLUMN locked_reason TEXT;

-- What zerver-admin did and to which account. No foreign key and no username or email, so a row
-- outlives the account it names, including one the admin deleted.
CREATE TABLE admin_action_log (
    id           BIGSERIAL PRIMARY KEY,
    account_id   UUID NOT NULL,
    action       TEXT NOT NULL,
    occurred_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_action_log_account ON admin_action_log(account_id, occurred_at DESC);
