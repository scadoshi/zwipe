-- Checks the rows already stored against the 1 to 99 range. Validating takes a
-- lock that still lets reads and writes through, and in its own migration it
-- never runs under the brief exclusive lock that adding the check took. The
-- range covers the older positive check, which goes.
ALTER TABLE deck_cards VALIDATE CONSTRAINT deck_cards_quantity_range;

ALTER TABLE deck_cards DROP CONSTRAINT positive_quantity;
