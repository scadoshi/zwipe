-- Widen daily counters to BIGINT so the ingest clamp is purely anti-abuse
-- and no longer doubles as i32-overflow protection (see the clamp note in
-- the metrics repository).
ALTER TABLE user_daily_activity
    ALTER COLUMN swipes_right TYPE BIGINT,
    ALTER COLUMN swipes_left  TYPE BIGINT,
    ALTER COLUMN swipes_up    TYPE BIGINT,
    ALTER COLUMN swipes_down  TYPE BIGINT,
    ALTER COLUMN searches     TYPE BIGINT;
