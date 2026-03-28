CREATE TABLE email_verification_tokens (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL,
    CONSTRAINT fk_user_evtoken
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_evt_user_id ON email_verification_tokens(user_id);
CREATE INDEX idx_evt_hash ON email_verification_tokens(token_hash);
CREATE INDEX idx_evt_expires_at ON email_verification_tokens(expires_at);
