-- The imports a client sent with a request_id, kept a day, so the same import sent again (its
-- reply lost on the way back) answers the first one's result instead of adding its copies twice.
-- body_sha256 hashes the mode and the file, so the same id with a different import is turned
-- down. Each import deletes its user's rows older than a day.
CREATE TABLE collection_import_requests (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    request_id UUID NOT NULL,
    body_sha256 BYTEA NOT NULL,
    added INTEGER NOT NULL,
    updated INTEGER NOT NULL,
    removed INTEGER NOT NULL,
    cards BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, request_id)
);
