CREATE TABLE decks (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    format VARCHAR NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES users (id)
        ON DELETE CASCADE
);

CREATE INDEX idx_decks_user_id ON decks(user_id);