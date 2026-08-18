-- MVPs are mainboard-only. `update_deck_card` already enforces this on both
-- sides (starring a non-mainboard card is rejected; moving a starred card off
-- the mainboard clears the star), but the import upsert set `board` on conflict
-- without touching `mvp_at`, so importing a starred card onto the maybeboard or
-- sideboard stranded the star there. That row then escaped the podium cap,
-- which counts mainboard stars only, letting a deck hold four.
--
-- Prod had no violations when this was written (36 MVPs, all mainboard); the
-- UPDATE is here so the constraint can't fail on a database that does.
UPDATE deck_cards SET mvp_at = NULL WHERE mvp_at IS NOT NULL AND board <> 'deck';

ALTER TABLE deck_cards
    ADD CONSTRAINT deck_cards_mvp_mainboard_only
    CHECK (mvp_at IS NULL OR board = 'deck');
