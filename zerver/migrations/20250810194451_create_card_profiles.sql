CREATE TABLE card_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scryfall_data_id UUID NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_scryfall_data
        FOREIGN KEY (scryfall_data_id)
        REFERENCES scryfall_data (id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_card_profiles_scryfall_data_id 
    ON card_profiles(scryfall_data_id);