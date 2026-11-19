-- A card's URL slug and, for a card with more than one face, its front face's
-- slug, which the card lookup by name reads. The sync writes both from
-- zwipe-core's card_slug; the next migrations index them and fill the rows
-- already there. Adding a column takes a lock that stops reads and writes until
-- the transaction ends, so this migration does nothing else: the columns are
-- nullable with no default, which rewrites nothing and ends at once. The lock
-- timeout gives up rather than queue every search behind a long-running query.
SET LOCAL lock_timeout = '5s';

ALTER TABLE scryfall_data ADD COLUMN IF NOT EXISTS name_slug text;
ALTER TABLE scryfall_data ADD COLUMN IF NOT EXISTS front_slug text;
