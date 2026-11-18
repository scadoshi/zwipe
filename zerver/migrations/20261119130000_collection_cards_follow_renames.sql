-- A printing's name, set code or collector number can change in the nightly card sync (Scryfall
-- renames a card, or moves a printing). The collection rows that hold that printing carry those
-- values for their sort and search, so a change on the printing rewrites them there too,
-- whichever writer made it. A sync that rewrites a printing with the same values changes nothing.
CREATE FUNCTION scryfall_data_refresh_collection() RETURNS trigger
    LANGUAGE plpgsql AS $$
BEGIN
    UPDATE collection_cards
    SET card_name = NEW.name,
        set_code = NEW.set,
        collector_number = NEW.collector_number,
        search_name = lower(f_unaccent(NEW.name))
    WHERE scryfall_data_id = NEW.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER scryfall_data_refresh_collection
    AFTER UPDATE OF name, set, collector_number ON scryfall_data
    FOR EACH ROW
    WHEN (OLD.name IS DISTINCT FROM NEW.name
          OR OLD.set IS DISTINCT FROM NEW.set
          OR OLD.collector_number IS DISTINCT FROM NEW.collector_number)
    EXECUTE FUNCTION scryfall_data_refresh_collection();
