-- Deck-level oracle tags: the granular functional tags (slugs from the
-- `oracle_tags` catalog) a deck declares as its strategy. Distinct from `tags`
-- (curated DeckTag archetypes) and `other_tags` (curated DeckOtherTag labels) --
-- this is a free `Vec<String>` of catalog slugs, mirroring `card_profiles`'
-- oracle-tag storage. Picking an archetype seeds these client-side.
-- See context/plans/otags/ (Phase 3).

ALTER TABLE decks ADD COLUMN oracle_tags JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX idx_decks_oracle_tags ON decks USING GIN(oracle_tags);
