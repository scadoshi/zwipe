-- Per-deck price target (budget). NULL = no budget set (no signal). Currency is
-- NULL = USD default. Additive nullable columns, safe to apply before the new
-- server binary ships.
ALTER TABLE decks ADD COLUMN price_target DOUBLE PRECISION;
ALTER TABLE decks ADD COLUMN price_target_currency TEXT;
