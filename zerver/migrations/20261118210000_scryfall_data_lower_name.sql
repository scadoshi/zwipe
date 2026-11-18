-- Finds printings by name without regard to case, as the collection import and deck import
-- match a row's name. Analyzed so the planner uses it from the first import.
CREATE INDEX IF NOT EXISTS idx_scryfall_data_lower_name ON scryfall_data (lower(name));
ANALYZE scryfall_data;
