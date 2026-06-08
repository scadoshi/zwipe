-- Convert deck timestamps to TIMESTAMPTZ. See the tokens migration for the
-- rationale behind USING ... AT TIME ZONE 'UTC'.

ALTER TABLE decks
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC',
    ALTER COLUMN first_completed_at TYPE TIMESTAMPTZ USING first_completed_at AT TIME ZONE 'UTC';

ALTER TABLE deck_cards
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';
