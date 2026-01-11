CREATE TABLE scryfall_data (
    -- Core Card Fields
    -- Cards have the following core properties
    id UUID PRIMARY KEY,
    arena_id INT,
    lang TEXT NOT NULL,
    mtgo_id INT UNIQUE,
    mtgo_foil_id INT UNIQUE,
    multiverse_ids INT[],
    tcgplayer_id INT,
    tcgplayer_etched_id INT,
    cardmarket_id INT,
    object TEXT NOT NULL,
    layout TEXT NOT NULL,
    oracle_id UUID,
    prints_search_uri TEXT NOT NULL,
    rulings_uri TEXT NOT NULL,
    scryfall_uri TEXT NOT NULL,
    uri TEXT NOT NULL,

    -- Gameplay Fields
    -- Cards have the following properties relevant to the game rules
    all_parts JSONB DEFAULT '[]'::jsonb,
    card_faces JSONB DEFAULT '[]'::jsonb,
    cmc FLOAT,
    color_identity TEXT[] DEFAULT ARRAY[]::TEXT[] NOT NULL,
    color_indicator TEXT[] DEFAULT ARRAY[]::TEXT[],
    colors TEXT[] DEFAULT ARRAY[]::TEXT[],
    defense TEXT,
    edhrec_rank INT,
    game_changer BOOLEAN,
    hand_modifier TEXT,
    keywords TEXT[],
    legalities JSONB NOT NULL,
    life_modifier TEXT,
    loyalty TEXT,
    mana_cost TEXT,
    name TEXT NOT NULL,
    oracle_text TEXT,
    penny_rank INT,
    power TEXT,
    produced_mana TEXT[],
    reserved BOOLEAN NOT NULL,
    toughness TEXT,
    type_line TEXT,

    -- Print Fields
    -- Cards have the following properties unique to their particular re/print
    artist TEXT,
    artist_ids UUID[],
    attraction_lights TEXT[],
    booster BOOLEAN NOT NULL,
    border_color TEXT NOT NULL,
    card_back_id UUID,
    collector_number TEXT NOT NULL,
    content_warning BOOLEAN,
    digital BOOLEAN NOT NULL,
    finishes TEXT[] NOT NULL,
    flavor_name TEXT,
    flavor_text TEXT,
    frame_effects TEXT[],
    frame TEXT NOT NULL,
    full_art BOOLEAN NOT NULL,
    games TEXT[],
    highres_image BOOLEAN NOT NULL,
    illustration_id UUID,
    image_status TEXT NOT NULL,
    image_uris JSONB,
    oversized BOOLEAN NOT NULL,
    prices JSONB NOT NULL,
    printed_name TEXT,
    printed_text TEXT,
    printed_type_line TEXT,
    promo BOOLEAN NOT NULL,
    promo_types TEXT[],
    purchase_uris JSONB,
    rarity TEXT NOT NULL,
    related_uris JSONB NOT NULL,
    released_at DATE NOT NULL,
    reprint BOOLEAN NOT NULL,
    scryfall_set_uri TEXT NOT NULL,
    set_name TEXT NOT NULL,
    set_search_uri TEXT NOT NULL,
    set_type TEXT NOT NULL,
    set_uri TEXT NOT NULL,
    set TEXT NOT NULL,
    set_id UUID NOT NULL,
    story_spotlight BOOLEAN NOT NULL,
    textless BOOLEAN NOT NULL,
    variation BOOLEAN NOT NULL,
    variation_of UUID,
    security_stamp TEXT,
    watermark TEXT,
    preview_previewed_at DATE,
    preview_source_uri TEXT,
    preview_source TEXT
);

CREATE INDEX idx_scryfall_data_name ON scryfall_data(name);
CREATE INDEX idx_scryfall_data_type ON scryfall_data(type_line);
CREATE INDEX idx_scryfall_data_rarity ON scryfall_data(rarity);
CREATE INDEX idx_scryfall_data_id ON scryfall_data(id);
CREATE INDEX idx_scryfall_data_set ON scryfall_data(set);
CREATE INDEX idx_scryfall_data_cmc ON scryfall_data(cmc);
CREATE INDEX idx_scryfall_data_colors ON scryfall_data USING GIN(colors);
CREATE INDEX idx_scryfall_data_oracle_id ON scryfall_data(oracle_id);
CREATE INDEX idx_scryfall_data_oracle_released ON scryfall_data(oracle_id, released_at DESC);
