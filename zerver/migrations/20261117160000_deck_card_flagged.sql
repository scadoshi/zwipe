-- Flag: a private on/off mark the deck's owner puts on a deck card. A constant
-- default adds the column without a backfill or a table rewrite; a removed card
-- takes its flag with its row, and a board move keeps it.
ALTER TABLE deck_cards ADD COLUMN flagged BOOLEAN NOT NULL DEFAULT false;
