-- Pre-computed deduplication: one row per unique card (latest printing per oracle_id).
-- Eliminates ROW_NUMBER() CTE from every search query.
-- Refreshed by zervice after sync + classification.
CREATE MATERIALIZED VIEW latest_cards AS
SELECT DISTINCT ON (COALESCE(sd.oracle_id, sd.id))
       sd.*
FROM scryfall_data sd
JOIN card_profiles cp ON sd.id = cp.scryfall_data_id
ORDER BY COALESCE(sd.oracle_id, sd.id), sd.released_at DESC
WITH NO DATA;

-- Unique index on id (required for future CONCURRENTLY refresh, useful for joins)
CREATE UNIQUE INDEX idx_latest_cards_id ON latest_cards(id);

-- Trigram GIN indexes: enable ILIKE '%term%' index scans
CREATE INDEX idx_latest_cards_name_trgm ON latest_cards USING GIN (name gin_trgm_ops);
CREATE INDEX idx_latest_cards_oracle_text_trgm ON latest_cards USING GIN (oracle_text gin_trgm_ops);
CREATE INDEX idx_latest_cards_type_line_trgm ON latest_cards USING GIN (type_line gin_trgm_ops);

-- GIN index for color_identity <@ operator
CREATE INDEX idx_latest_cards_color_identity ON latest_cards USING GIN (color_identity);

-- B-tree indexes for equality, range, and sort filters
CREATE INDEX idx_latest_cards_name ON latest_cards(name);
CREATE INDEX idx_latest_cards_cmc ON latest_cards(cmc);
CREATE INDEX idx_latest_cards_rarity ON latest_cards(rarity);
CREATE INDEX idx_latest_cards_set_name ON latest_cards(set_name);
CREATE INDEX idx_latest_cards_lang ON latest_cards(lang);
