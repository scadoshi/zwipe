CREATE TABLE cards (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    mana_cost VARCHAR,
    card_type VARCHAR NOT NULL,
    rarity VARCHAR NOT NULL,
    image_url VARCHAR NOT NULL,
    oracle_text VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cards_name ON cards(name);
CREATE INDEX idx_cards_type ON cards(card_type);
CREATE INDEX idx_cards_rarity ON cards(rarity);
