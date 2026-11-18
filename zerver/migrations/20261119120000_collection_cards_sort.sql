-- The collection list's sort and search columns, stored on each row so a list, a page after a
-- cursor and a name search all read one user's rows through one index instead of walking the
-- card catalog: the card's name, its set code and collector number, and the name folded for
-- search (lower case, accents off). Filled here from the catalog, then by a trigger on every row
-- written with a printing (the import, the set call, anything else), so they can't drift from
-- the printing a row names.
ALTER TABLE collection_cards
    ADD COLUMN card_name TEXT,
    ADD COLUMN set_code TEXT,
    ADD COLUMN collector_number TEXT,
    ADD COLUMN search_name TEXT;

UPDATE collection_cards cc
SET card_name = sd.name,
    set_code = sd.set,
    collector_number = sd.collector_number,
    search_name = lower(f_unaccent(sd.name))
FROM scryfall_data sd
WHERE sd.id = cc.scryfall_data_id;

CREATE FUNCTION collection_cards_fill_sort() RETURNS trigger
    LANGUAGE plpgsql AS $$
BEGIN
    SELECT sd.name, sd.set, sd.collector_number, lower(f_unaccent(sd.name))
    INTO NEW.card_name, NEW.set_code, NEW.collector_number, NEW.search_name
    FROM scryfall_data sd
    WHERE sd.id = NEW.scryfall_data_id;
    RETURN NEW;
END
$$;

CREATE TRIGGER collection_cards_fill_sort
    BEFORE INSERT OR UPDATE OF scryfall_data_id ON collection_cards
    FOR EACH ROW EXECUTE FUNCTION collection_cards_fill_sort();

ALTER TABLE collection_cards
    ALTER COLUMN card_name SET NOT NULL,
    ALTER COLUMN set_code SET NOT NULL,
    ALTER COLUMN collector_number SET NOT NULL,
    ALTER COLUMN search_name SET NOT NULL;

CREATE INDEX collection_cards_list ON collection_cards (user_id, card_name, set_code);
