-- The card lookup's indexes, built while every slug is still NULL so the build
-- is short. A plain build holds a lock that lets every read through and makes
-- writes wait, and only the nightly sync writes this table; the backfill's
-- updates keep them current. In a transaction, a failed build leaves nothing
-- behind to clean up.
SET LOCAL lock_timeout = '5s';

CREATE INDEX IF NOT EXISTS idx_scryfall_data_name_slug ON scryfall_data (name_slug);
CREATE INDEX IF NOT EXISTS idx_scryfall_data_front_slug ON scryfall_data (front_slug);
