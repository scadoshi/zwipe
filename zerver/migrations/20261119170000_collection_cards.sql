-- A user's card collection: one row per printing, finish, condition and language, private to
-- its user. Ownership is read by oracle id, so a card counts as owned whichever printing is held.
CREATE TABLE collection_cards (
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    scryfall_data_id UUID NOT NULL REFERENCES scryfall_data (id),
    oracle_id UUID NOT NULL,
    finish TEXT NOT NULL CHECK (finish IN ('nonfoil', 'foil', 'etched')),
    condition TEXT CHECK (condition IN (
        'mint', 'near_mint', 'excellent', 'good', 'lightly_played',
        'moderately_played', 'played', 'heavily_played', 'poor', 'damaged'
    )),
    language TEXT CHECK (language IN (
        'en', 'es', 'fr', 'de', 'it', 'pt', 'ja', 'ko', 'ru',
        'zhs', 'zht', 'he', 'la', 'grc', 'ar', 'sa', 'ph', 'qya'
    )),
    quantity INT NOT NULL CHECK (quantity BETWEEN 1 AND 9999),
    tags TEXT[] NOT NULL DEFAULT '{}' CHECK (cardinality(tags) <= 20),
    purchase_price NUMERIC(12, 2) CHECK (purchase_price >= 0),
    purchase_currency TEXT CHECK (purchase_currency ~ '^[A-Z]{3}$'),
    added_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One row per printing, finish, condition and language; a missing condition or language is one
-- value of its own, not a wildcard.
CREATE UNIQUE INDEX collection_cards_key
    ON collection_cards (user_id, scryfall_data_id, finish, condition, language) NULLS NOT DISTINCT;

-- Whether a user owns a card, by oracle id.
CREATE INDEX collection_cards_owned ON collection_cards (user_id, oracle_id);

-- The printing removal sweep finds the rows that hold a printing.
CREATE INDEX collection_cards_printing ON collection_cards (scryfall_data_id);
