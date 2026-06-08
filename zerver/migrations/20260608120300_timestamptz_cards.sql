-- Convert card-profile and sync-metrics timestamps to TIMESTAMPTZ. See the
-- tokens migration for the rationale behind USING ... AT TIME ZONE 'UTC'.
--
-- scryfall_data.released_at is intentionally left as-is: it is a property of
-- the printing (a release date), not an instant our system recorded, so the
-- TIMESTAMPTZ semantic does not apply. Revisit with the Scryfall sync.

ALTER TABLE card_profiles
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

ALTER TABLE zervice_metrics
    ALTER COLUMN started_at TYPE TIMESTAMPTZ USING started_at AT TIME ZONE 'UTC',
    ALTER COLUMN ended_at   TYPE TIMESTAMPTZ USING ended_at   AT TIME ZONE 'UTC';
