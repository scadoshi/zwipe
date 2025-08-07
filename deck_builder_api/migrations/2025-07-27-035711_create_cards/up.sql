CREATE TABLE cards (
    -- My Fields
    -- Those that exist only in my database
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()

    -- Core Card Fields
    -- Cards have the following core properties
    arena_id INT,
    scryfall_id UUID NOT NULL,
    lang VARCHAR NOT NULL,
    mtgo_id INT,
    mtgo_foil_id INT,
    multiverse_ids INT[],
    tcgplayer_id INT,
    tcgplayer_etched_id INT,
    cardmarket_id INT,
    object VARCHAR NOT NULL,
    layout VARCHAR NOT NULL,
    oracle_id UUID,
    prints_search_uri VARCHAR NOT NULL,
    rulings_uri VARCHAR NOT NULL,
    scryfall_uri VARCHAR NOT NULL,
    scryfall_api_uri VARCHAR NOT NULL,

    -- Gameplay Fields
    -- Cards have the following properties relevant to the game rules
    all_parts JSONB,
    card_faces JSONB[],
    cmc FLOAT NOT NULL,
    color_identity VARCHAR[],
    color_indicator VARCHAR[],
    colors VARCHAR[],
    defense VARCHAR,
    edhrec_rank INT,
    game_changer BOOLEAN,
    hand_modifier VARCHAR,
    keywords VARCHAR[],
    legalities JSONB,
    life_modifier VARCHAR,
    loyalty VARCHAR,
    mana_cost VARCHAR,
    name VARCHAR NOT NULL,
    oracle_text VARCHAR,
    penny_rank INT,
    power VARCHAR,
    produced_mana JSONB,
    reserved BOOLEAN NOT NULL,
    toughness VARCHAR,
    type_line VARCHAR NOT NULL

    -- Print Fields
    -- Cards have the following properties unique to their particular re/print
    artist VARCHAR,
    artist_ids UUID[],
    attraction_lights VARCHAR[],
    booster BOOLEAN NOT NULL,
    border_color VARCHAR NOT NULL,
    card_back_id UUID NOT NULL,
    collector_number VARCHAR NOT NULL,
    content_warning BOOLEAN,
    digital BOOLEAN NOT NULL,
    finishes VARCHAR[] NOT NULL,
    flavor_name VARCHAR,
    flavor_text VARCHAR,
    frame_effects VARCHAR[],
    frame VARCHAR NOT NULL,
    full_art BOOLEAN NOT NULL,
    games VARCHAR[],
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
    promo_types VARCHAR[],
    purchase_uris JSONB,
    rarity VARCHAR NOT NULL,
    related_uris JSONB NOT NULL,
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

    -- Card Face Objects
    -- Multiface cards have a card_faces property containing at least two Card Face objects. Those objects have the following properties

    -- 
    -- 

);

CREATE INDEX idx_cards_name ON cards(name);
CREATE INDEX idx_cards_type ON cards(type_line);
CREATE INDEX idx_cards_rarity ON cards(rarity);
