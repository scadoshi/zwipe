-- MVP podium (context/plans/deck_mvps/): up to 3 starred cards per deck. The
-- timestamp is the vesting clock (global signal counts it after 3 days);
-- NULL = not an MVP. Cap and board rules are enforced in the update handler,
-- not the schema.
ALTER TABLE deck_cards ADD COLUMN mvp_at TIMESTAMPTZ;
