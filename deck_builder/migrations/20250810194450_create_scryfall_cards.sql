CREATE TABLE scryfall_cards (
    -- Core Card Fields
    -- Cards have the following core properties
    id UUID PRIMARY KEY,
    arena_id INT,
    lang VARCHAR NOT NULL,
    mtgo_id INT UNIQUE,
    mtgo_foil_id INT UNIQUE,
    multiverse_ids INT[],
    tcgplayer_id INT,
    tcgplayer_etched_id INT UNIQUE,
    cardmarket_id INT UNIQUE,
    object VARCHAR NOT NULL,
    layout VARCHAR NOT NULL,
    oracle_id UUID,
    prints_search_uri VARCHAR NOT NULL,
    rulings_uri VARCHAR NOT NULL,
    scryfall_uri VARCHAR NOT NULL,
    uri VARCHAR NOT NULL,

    -- Gameplay Fields
    -- Cards have the following properties relevant to the game rules
    all_parts JSONB[],
    card_faces JSONB[],
    cmc FLOAT NOT NULL,
    color_identity TEXT[] NOT NULL,
    color_indicator TEXT[],
    colors TEXT[],
    defense VARCHAR,
    edhrec_rank INT,
    game_changer BOOLEAN,
    hand_modifier VARCHAR,
    keywords TEXT[],
    legalities JSONB NOT NULL,
    life_modifier VARCHAR,
    loyalty VARCHAR,
    mana_cost VARCHAR,
    name VARCHAR NOT NULL,
    oracle_text VARCHAR,
    penny_rank INT,
    power VARCHAR,
    produced_mana TEXT[],
    reserved BOOLEAN NOT NULL,
    toughness VARCHAR,
    type_line VARCHAR NOT NULL,

    -- Print Fields
    -- Cards have the following properties unique to their particular re/print
    artist VARCHAR,
    artist_ids UUID[],
    attraction_lights TEXT[],
    booster BOOLEAN NOT NULL,
    border_color VARCHAR NOT NULL,
    card_back_id UUID,
    collector_number VARCHAR NOT NULL,
    content_warning BOOLEAN,
    digital BOOLEAN NOT NULL,
    finishes TEXT[] NOT NULL,
    flavor_name VARCHAR,
    flavor_text VARCHAR,
    frame_effects TEXT[],
    frame VARCHAR NOT NULL,
    full_art BOOLEAN NOT NULL,
    games TEXT[],
    highres_image BOOLEAN NOT NULL,
    illustration_id UUID,
    image_status VARCHAR NOT NULL,
    image_uris JSONB,
    oversized BOOLEAN NOT NULL,
    prices JSONB NOT NULL,
    printed_name VARCHAR,
    printed_text VARCHAR,
    printed_type_line VARCHAR,
    promo BOOLEAN NOT NULL,
    promo_types TEXT[],
    -- purchase_uris JSONB, (*3*)
    rarity VARCHAR NOT NULL,
    -- related_uris JSONB NOT NULL, (*3*)
    released_at DATE NOT NULL,
    reprint BOOLEAN NOT NULL,
    scryfall_set_uri VARCHAR NOT NULL,
    set_name VARCHAR NOT NULL,
    set_search_uri VARCHAR NOT NULL,
    set_type VARCHAR NOT NULL,
    set_uri VARCHAR NOT NULL,
    set VARCHAR NOT NULL,
    set_id UUID NOT NULL,
    story_spotlight BOOLEAN NOT NULL,
    textless BOOLEAN NOT NULL,
    variation BOOLEAN NOT NULL,
    variation_of UUID,
    security_stamp VARCHAR,
    watermark VARCHAR,
    preview_previewed_at DATE,
    preview_source_uri VARCHAR,
    preview_source VARCHAR
);

CREATE INDEX idx_cards_name ON scryfall_cards(name);
CREATE INDEX idx_cards_type ON scryfall_cards(type_line);
CREATE INDEX idx_cards_rarity ON scryfall_cards(rarity);
CREATE INDEX idx_cards_id ON scryfall_cards(id);
CREATE INDEX idx_cards_set ON scryfall_cards(set);
CREATE INDEX idx_cards_cmc ON scryfall_cards(cmc);
CREATE INDEX idx_cards_colors ON scryfall_cards USING GIN(colors);