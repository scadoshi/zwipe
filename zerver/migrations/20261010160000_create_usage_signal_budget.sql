-- Per-user daily budget for the pooled card and commander signals: how many
-- impressions (`shown`) a user's flushes have contributed today. Ingest adds
-- a flush's total here first and drops the flush's signals once the day's
-- total passes the cap, so no one account can steer the pooled ranking.
-- Rows older than two days are deleted by ingest itself.
CREATE TABLE usage_signal_budget (
    user_id UUID   NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    day     DATE   NOT NULL,
    shown   BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, day)
);

CREATE INDEX idx_usage_signal_budget_day ON usage_signal_budget (day);
