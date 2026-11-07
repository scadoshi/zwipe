-- The public totals count everything ever done on Zwipe. A deleted account's
-- lifetime counters fold into this one row as they go, so the totals never
-- drop; the per-user row still leaves with the account.
CREATE TABLE retired_lifetime_counters (
    id               BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    swipes_right     BIGINT NOT NULL DEFAULT 0,
    swipes_left      BIGINT NOT NULL DEFAULT 0,
    swipes_up        BIGINT NOT NULL DEFAULT 0,
    swipes_down      BIGINT NOT NULL DEFAULT 0,
    searches         BIGINT NOT NULL DEFAULT 0,
    decks_created    BIGINT NOT NULL DEFAULT 0,
    decks_completed  BIGINT NOT NULL DEFAULT 0
);

INSERT INTO retired_lifetime_counters DEFAULT VALUES;

CREATE FUNCTION retire_lifetime_counters() RETURNS trigger AS $$
BEGIN
    UPDATE retired_lifetime_counters SET
        swipes_right    = swipes_right    + OLD.swipes_right,
        swipes_left     = swipes_left     + OLD.swipes_left,
        swipes_up       = swipes_up       + OLD.swipes_up,
        swipes_down     = swipes_down     + OLD.swipes_down,
        searches        = searches        + OLD.searches,
        decks_created   = decks_created   + OLD.decks_created,
        decks_completed = decks_completed + OLD.decks_completed;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Fires for the cascade from users too, so deleting an account retires its
-- counters with no code of its own.
CREATE TRIGGER user_lifetime_counters_retire
    BEFORE DELETE ON user_lifetime_counters
    FOR EACH ROW EXECUTE FUNCTION retire_lifetime_counters();
