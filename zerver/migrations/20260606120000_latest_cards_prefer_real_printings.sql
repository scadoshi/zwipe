-- Rebuild latest_cards so DISTINCT ON picks the most "normal" printing per oracle_id
-- (paper before digital, non-promo before promo, standard size, non-content-warning),
-- then falls back to most recent release. Previously the view picked latest-released
-- only, surfacing MTGA-only rows for cards like Wear // Tear and promo-flagged rows
-- for cards with Secret Lair / Jumpstart printings.

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

REFRESH MATERIALIZED VIEW latest_cards;

-- Remap existing deck and deck_card references whose scryfall_data_id points at a
-- non-preferred sibling printing (same oracle_id, but the view now selects a
-- different row). Skips rows already on the preferred printing and rows with no
-- oracle_id (unique cards have no sibling).

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
