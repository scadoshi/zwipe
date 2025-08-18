CREATE TABLE deck_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
        ON DELETE CASCADE
);

CREATE INDEX idx_deck_cards_card_id ON deck_cards(card_profile_id);
CREATE INDEX idx_deck_cards_deck_id ON deck_cards(deck_id);