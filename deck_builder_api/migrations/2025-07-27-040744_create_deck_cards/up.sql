CREATE TABLE deck_cards (
    id SERIAL PRIMARY KEY,
    deck_id INT NOT NULL,
    card_id INT NOT NULL,
    quantity INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_deck
        FOREIGN KEY (deck_id)
        REFERENCES decks (id)
        ON DELETE CASCADE,
    CONSTRAINT fk_card
        FOREIGN KEY (card_id)
        REFERENCES cards (id)
        ON DELETE CASCADE
);

CREATE INDEX idx_deck_cards_card_id ON deck_cards(card_id);
CREATE INDEX idx_deck_cards_deck_id ON deck_cards(deck_id);