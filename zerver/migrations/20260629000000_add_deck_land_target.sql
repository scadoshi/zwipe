-- Per-deck land target. NULL = use the format-derived heuristic; a value is the
-- user's explicit override. Additive nullable column, safe to apply before the
-- new server binary ships.
ALTER TABLE decks ADD COLUMN land_target INTEGER;
