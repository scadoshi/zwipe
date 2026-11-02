-- Usage totals per UTC day per client platform, with no user id. Each usage
-- flush adds its capped counts to the row for its access token's platform
-- ('ios', 'android', 'desktop', 'web', 'terminal', or 'unknown' for a token
-- minted without one); flushes counts the batches.
CREATE TABLE platform_daily_usage (
    day          DATE   NOT NULL,
    platform     TEXT   NOT NULL,
    swipes_right BIGINT NOT NULL DEFAULT 0,
    swipes_left  BIGINT NOT NULL DEFAULT 0,
    swipes_up    BIGINT NOT NULL DEFAULT 0,
    swipes_down  BIGINT NOT NULL DEFAULT 0,
    searches     BIGINT NOT NULL DEFAULT 0,
    flushes      BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (day, platform)
);
