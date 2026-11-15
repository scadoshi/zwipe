-- Printings Scryfall deleted, which zervice removed after moving every deck
-- reference to replacement_id (zwipe-context ideas/sync_remove_deleted_printings.md).
-- replacement_id is NULL only for a printing whose card had no other printing
-- and that nothing referenced. zerver reads it to translate an id an older
-- client still holds onto the printing that replaced it.
--
-- Additive only: a new table nothing references, so the running zerver keeps
-- working unchanged until its new binary ships.
CREATE TABLE IF NOT EXISTS removed_printings (
    id             UUID        PRIMARY KEY,
    oracle_id      UUID,
    replacement_id UUID,
    removed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- zervice applies the removal, so it gets the log in full and, on deck_cards
-- and decks, only the printing id columns it reads and moves: it still sees
-- no deck name, owner, quantity or board, and nothing about users
-- (context/archive/zervice_least_privilege.md). zcripts/server/sql/zervice_role.sql
-- holds the same grants. A test database has no such role.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'zervice') THEN
        GRANT SELECT, INSERT, UPDATE, DELETE ON removed_printings TO zervice;
        GRANT SELECT (scryfall_data_id), UPDATE (scryfall_data_id) ON deck_cards TO zervice;
        GRANT SELECT (commander_id, partner_commander_id, background_id, signature_spell_id),
              UPDATE (commander_id, partner_commander_id, background_id, signature_spell_id)
              ON decks TO zervice;
    END IF;
END $$;
