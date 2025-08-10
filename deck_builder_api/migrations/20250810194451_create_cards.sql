CREATE TABLE cards (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    scryfall_id UUID NOT NULL UNIQUE,
    CONSTRAINT fk_scryfall_card
        FOREIGN KEY (scryfall_id)
        REFERENCES scryfall_cards (id)
        ON DELETE CASCADE
);