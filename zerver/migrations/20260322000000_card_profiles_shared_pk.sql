-- Promote scryfall_data_id to primary key, removing the surrogate id column.
--
-- card_profiles.id was a surrogate UUID that served no purpose beyond existing:
-- every lookup, route, and API call already used scryfall_data_id as the real
-- identifier. This migration collapses the two columns into one, making
-- scryfall_data_id both the PK and the FK to scryfall_data — a standard
-- shared-primary-key pattern for a strict 1:1 relationship.

ALTER TABLE card_profiles DROP COLUMN id;

-- scryfall_data_id already had a UNIQUE constraint; promote it to PRIMARY KEY.
ALTER TABLE card_profiles ADD PRIMARY KEY (scryfall_data_id);

-- The explicit index is now redundant — the PK index covers it.
DROP INDEX IF EXISTS idx_card_profiles_scryfall_data_id;
