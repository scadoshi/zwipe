-- Public share link: NULL = private (default). The token is the capability:
-- unguessable UUID, revoked by nulling, rotated by regenerating.
ALTER TABLE decks ADD COLUMN share_token UUID;
CREATE UNIQUE INDEX idx_decks_share_token ON decks (share_token)
    WHERE share_token IS NOT NULL;
