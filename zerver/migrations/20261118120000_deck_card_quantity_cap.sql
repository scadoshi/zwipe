-- A deck card holds 1 to 99 copies. Counts above 99 come down to 99, and the
-- check holds every row written from here on. It is added NOT VALID, so adding
-- it takes only a brief lock and never scans the table; the next migration
-- validates the rows already there.
UPDATE deck_cards SET quantity = 99 WHERE quantity > 99;

ALTER TABLE deck_cards
    ADD CONSTRAINT deck_cards_quantity_range CHECK (quantity BETWEEN 1 AND 99) NOT VALID;
