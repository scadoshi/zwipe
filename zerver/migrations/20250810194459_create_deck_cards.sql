CREATE TABLE deck_cards (
    deck_id UUID NOT NULL,
    scryfall_data_id UUID NOT NULL,
    quantity INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_deck
        FOREIGN KEY (deck_id)
        REFERENCES decks (id)
        ON DELETE CASCADE,
    CONSTRAINT fk_scryfall_data_id
        FOREIGN KEY (scryfall_data_id)
        REFERENCES scryfall_data (id)
        ON DELETE CASCADE,
    CONSTRAINT deck_card_unique UNIQUE (deck_id, scryfall_data_id),
    CONSTRAINT positive_quantity CHECK (quantity > 0)
);
