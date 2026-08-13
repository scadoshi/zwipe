-- Rebuild latest_cards so the DISTINCT ON pick prefers English printings ahead
-- of recency. The ordering never considered language, so a newer foreign
-- printing shadowed every English one and the app's default language=en filter
-- made the card vanish from search entirely: 268 cards as of 2026-08-12, e.g.
-- Arcane Signet behind hoc's Dwarvish (dw) spoilers and a tail of Japanese-only
-- soa reprints. Foreign-ONLY cards are unaffected: with no English printing
-- every row ties on the new key and recency still decides.

DROP MATERIALIZED VIEW IF EXISTS latest_cards;

CREATE MATERIALIZED VIEW latest_cards AS
SELECT DISTINCT ON (COALESCE(sd.oracle_id, sd.id))
       sd.*
FROM scryfall_data sd
JOIN card_profiles cp ON sd.id = cp.scryfall_data_id
ORDER BY COALESCE(sd.oracle_id, sd.id),
         (sd.digital) ASC,
         (sd.promo) ASC,
         (sd.oversized) ASC,
         (COALESCE(sd.content_warning, false)) ASC,
         (sd.lang IS DISTINCT FROM 'en') ASC,
         sd.released_at DESC
WITH NO DATA;

CREATE UNIQUE INDEX idx_latest_cards_id ON latest_cards(id);
CREATE INDEX idx_latest_cards_name_trgm ON latest_cards USING GIN (name gin_trgm_ops);
CREATE INDEX idx_latest_cards_oracle_text_trgm ON latest_cards USING GIN (oracle_text gin_trgm_ops);
CREATE INDEX idx_latest_cards_type_line_trgm ON latest_cards USING GIN (type_line gin_trgm_ops);
CREATE INDEX idx_latest_cards_color_identity ON latest_cards USING GIN (color_identity);
CREATE INDEX idx_latest_cards_name ON latest_cards(name);
CREATE INDEX idx_latest_cards_cmc ON latest_cards(cmc);
CREATE INDEX idx_latest_cards_rarity ON latest_cards(rarity);
CREATE INDEX idx_latest_cards_set_name ON latest_cards(set_name);
CREATE INDEX idx_latest_cards_lang ON latest_cards(lang);
-- Composite (edhrec_rank, name) matching 20260630010000 — the DROP above
-- destroys it, so the rebuild recreates it with the same shape.
CREATE INDEX idx_latest_cards_edhrec_rank ON latest_cards (edhrec_rank, name);

REFRESH MATERIALIZED VIEW latest_cards;

-- Remap existing deck and deck_card references whose scryfall_data_id points at
-- a printing the view no longer selects (same oracle_id, different preferred
-- row) — the same hygiene pass as the 2026-06-06 rebuild. Users who added a
-- shadowed card picked up the foreign printing id (soa's Japanese rows have
-- been the pick since April); this points them back at the English printing.

WITH remap AS (
    SELECT sd.id AS old_id, lc.id AS new_id
    FROM scryfall_data sd
    JOIN latest_cards lc ON lc.oracle_id = sd.oracle_id
    WHERE sd.oracle_id IS NOT NULL
      AND sd.id <> lc.id
)
UPDATE deck_cards dc
SET scryfall_data_id = r.new_id
FROM remap r
WHERE dc.scryfall_data_id = r.old_id;

WITH remap AS (
    SELECT sd.id AS old_id, lc.id AS new_id
    FROM scryfall_data sd
    JOIN latest_cards lc ON lc.oracle_id = sd.oracle_id
    WHERE sd.oracle_id IS NOT NULL
      AND sd.id <> lc.id
)
UPDATE decks d
SET commander_id = r.new_id
FROM remap r
WHERE d.commander_id = r.old_id;

WITH remap AS (
    SELECT sd.id AS old_id, lc.id AS new_id
    FROM scryfall_data sd
    JOIN latest_cards lc ON lc.oracle_id = sd.oracle_id
    WHERE sd.oracle_id IS NOT NULL
      AND sd.id <> lc.id
)
UPDATE decks d
SET partner_commander_id = r.new_id
FROM remap r
WHERE d.partner_commander_id = r.old_id;

WITH remap AS (
    SELECT sd.id AS old_id, lc.id AS new_id
    FROM scryfall_data sd
    JOIN latest_cards lc ON lc.oracle_id = sd.oracle_id
    WHERE sd.oracle_id IS NOT NULL
      AND sd.id <> lc.id
)
UPDATE decks d
SET background_id = r.new_id
FROM remap r
WHERE d.background_id = r.old_id;

WITH remap AS (
    SELECT sd.id AS old_id, lc.id AS new_id
    FROM scryfall_data sd
    JOIN latest_cards lc ON lc.oracle_id = sd.oracle_id
    WHERE sd.oracle_id IS NOT NULL
      AND sd.id <> lc.id
)
UPDATE decks d
SET signature_spell_id = r.new_id
FROM remap r
WHERE d.signature_spell_id = r.old_id;
