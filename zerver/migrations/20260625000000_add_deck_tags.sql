-- Deck-level archetype/strategy tags, stored as a JSONB array of snake_case
-- DeckTag strings (mirrors card_profiles.mechanical_categories).
ALTER TABLE decks ADD COLUMN tags JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX idx_decks_tags ON decks USING GIN(tags);
