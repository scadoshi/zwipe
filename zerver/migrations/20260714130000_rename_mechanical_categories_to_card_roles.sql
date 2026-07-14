-- Phase M sunset: the coarse role axis is `card_roles` everywhere now (the wire
-- field and the `mechanical_categories_*` criteria alias were dropped). Rename the
-- storage column and its GIN index to match. Data-preserving rename; all queries
-- are updated in lockstep, so there is no window where a query names the old column.
ALTER TABLE card_profiles RENAME COLUMN mechanical_categories TO card_roles;
ALTER INDEX idx_card_profiles_categories RENAME TO idx_card_profiles_card_roles;
