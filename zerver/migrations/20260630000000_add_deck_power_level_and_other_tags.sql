-- Secondary deck-profile axes, both additive and safe to apply before the new
-- server binary ships.
--   power_level: the WotC Commander Bracket (snake_case string). NULL = unset.
--   other_tags : non-gameplay labels (Budget/Jank/…), a JSONB array of
--                snake_case strings, mirroring decks.tags.
ALTER TABLE decks ADD COLUMN power_level TEXT;
ALTER TABLE decks ADD COLUMN other_tags JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX idx_decks_other_tags ON decks USING GIN(other_tags);
