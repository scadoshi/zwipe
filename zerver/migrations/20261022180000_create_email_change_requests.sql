-- One pending email change per user, held until the link sent to the new
-- address is clicked. No unique constraint on new_email: users.email decides
-- uniqueness at confirm time.
CREATE TABLE email_change_requests (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    new_email TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_ecr_expires_at ON email_change_requests(expires_at);
