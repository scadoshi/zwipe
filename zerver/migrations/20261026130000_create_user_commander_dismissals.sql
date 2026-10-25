-- Commander dismissals for suggested commanders
-- (context/plans/suggested_commanders.md).
--
-- One row per commander a user turned down from the suggestions. A row hides
-- the commander while dismissed_at is within the last 7 days (zerver's
-- DISMISSAL_TTL_DAYS); dismissing it again refreshes dismissed_at. Keyed by
-- oracle_id so a dismissal covers every printing. Cascade: account delete.

CREATE TABLE user_commander_dismissals (
    user_id      uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    oracle_id    uuid        NOT NULL,
    dismissed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, oracle_id)
);
