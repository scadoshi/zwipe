-- A deck card holds 1 to 99 copies. The check comes first, so a write over 99
-- is refused from that moment, even one from a server still running the older
-- code; then counts above 99 come down to 99. It is added NOT VALID, so adding
-- it never scans the table; the next migration validates the rows already
-- there.
ALTER TABLE deck_cards
    ADD CONSTRAINT deck_cards_quantity_range CHECK (quantity BETWEEN 1 AND 99) NOT VALID;

UPDATE deck_cards SET quantity = 99 WHERE quantity > 99;
