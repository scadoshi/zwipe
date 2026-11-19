-- A card's URL slug and, for a card with more than one face, its front face's
-- slug, which the card lookup by name reads, each indexed. The sync writes
-- both from zwipe-core's card_slug; the next migration fills the rows already
-- there. The columns are nullable with no default, so adding them rewrites
-- nothing, and the indexes are built while every value is still NULL, so the
-- lock they hold (reads go on, writes wait) lasts a moment; the backfill's
-- updates keep them current. The lock timeout gives up rather than queue
-- every search behind a long-running query, and a failure rolls back whole.
SET LOCAL lock_timeout = '5s';

ALTER TABLE scryfall_data ADD COLUMN IF NOT EXISTS name_slug text;
ALTER TABLE scryfall_data ADD COLUMN IF NOT EXISTS front_slug text;

CREATE INDEX IF NOT EXISTS idx_scryfall_data_name_slug ON scryfall_data (name_slug);
CREATE INDEX IF NOT EXISTS idx_scryfall_data_front_slug ON scryfall_data (front_slug);
