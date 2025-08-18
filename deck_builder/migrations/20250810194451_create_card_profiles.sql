CREATE TABLE card_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scryfall_card_id UUID NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_scryfall_card
        FOREIGN KEY (scryfall_card_id)
        REFERENCES scryfall_cards (id)
        ON DELETE CASCADE
);