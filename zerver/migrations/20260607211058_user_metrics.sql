ALTER TABLE decks
    ADD COLUMN first_completed_at TIMESTAMP;

CREATE TABLE user_lifetime_counters (
    user_id          UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    swipes_right     BIGINT NOT NULL DEFAULT 0,
    swipes_left      BIGINT NOT NULL DEFAULT 0,
    swipes_up        BIGINT NOT NULL DEFAULT 0,
    swipes_down      BIGINT NOT NULL DEFAULT 0,
    searches         BIGINT NOT NULL DEFAULT 0,
    decks_created    INTEGER NOT NULL DEFAULT 0,
    decks_completed  INTEGER NOT NULL DEFAULT 0,
    updated_at       TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE user_daily_activity (
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    day           DATE NOT NULL,
    swipes_right  INTEGER NOT NULL DEFAULT 0,
    swipes_left   INTEGER NOT NULL DEFAULT 0,
    swipes_up     INTEGER NOT NULL DEFAULT 0,
    swipes_down   INTEGER NOT NULL DEFAULT 0,
    searches      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, day)
);

CREATE INDEX idx_user_daily_activity_day ON user_daily_activity(day);

CREATE TABLE user_events (
    id           BIGSERIAL PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind         TEXT NOT NULL,
    deck_id      UUID REFERENCES decks(id) ON DELETE SET NULL,
    occurred_at  TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_events_user ON user_events(user_id, occurred_at DESC);
CREATE INDEX idx_user_events_kind ON user_events(kind, occurred_at DESC);

CREATE TABLE user_audit_log (
    id           BIGSERIAL PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action       TEXT NOT NULL,
    occurred_at  TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_audit_log_user ON user_audit_log(user_id, occurred_at DESC);

INSERT INTO user_lifetime_counters (user_id)
SELECT id FROM users
ON CONFLICT (user_id) DO NOTHING;
