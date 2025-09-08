CREATE TABLE deck_cards (
    deck_id UUID NOT NULL,
    card_profile_id UUID NOT NULL,
    quantity INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_deck
        FOREIGN KEY (deck_id)
        REFERENCES decks (id)
        ON DELETE CASCADE,
    CONSTRAINT fk_card_profile
        FOREIGN KEY (card_profile_id)
        REFERENCES card_profiles (id)
        ON DELETE CASCADE,
    CONSTRAINT deck_card_unique UNIQUE (deck_id, card_profile_id),
    CONSTRAINT positive_quantity CHECK (quantity > 0)
);