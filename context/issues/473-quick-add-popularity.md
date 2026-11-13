# 473. Quick add offers obscure cards before popular ones within a name tier

State: fixing | Owner: thinker | Type: feature | PR: - (pushed to release) | Build seen: e3d5981fa

## Report
Scotty, via the composer: Quick add should show the most popular card first within each name tier, by EDHREC rank, so "lightning" offers Lightning Bolt in its top 5. Order: the name tiers, then `edhrec_rank` NULLS LAST, then name. Only Quick add's order changes.

## Cause
- Quick add (`zwipe_ui::screens::deck_cards::quick_add_query`) asks for the Name sort, so the server (`zerver/src/lib/outbound/sqlx/card/mod.rs`, the ORDER BY's `push_tiers`) orders each tier alphabetically. "lightning" fills its five with Lightning Angel, Lightning Axe and the rest of the first-word tier before Lightning Bolt.
- The request is the same as Search's Name sort with a name query, so the server can't change one without the other.

## Fix
Phased per `development/api_evolution.md`, since an older server's `Json<CardQuery>` refuses an unknown sort key:
- **A (server and core):** `CardSortKey::BestMatch`: the name tiers, then `edhrec_rank ASC NULLS LAST`, then name, best first whatever the direction. `CardSortKey::all()` leaves it out, so no sort picker offers it. No client sends it yet.
- **B (clients, after the server is live):** `quick_add_query` sends `BestMatch`, on the web and in the app.

Checked before A:
- The sort key travels in the JSON body of the search POSTs (`SearchCards`, `SearchDeckCards`, `SearchCommanders`); no response carries one.
- Shipped sort pickers (`zwipe_ui::filter::chips`, the app's `filter/sort.rs`) list the client's own compiled `CardSortKey::all()`, so they never see a new variant.

## Tests
- `zerver/tests/card_serving.rs` `best_match_ranks_each_name_tier_by_popularity`: "lightning" gives Lightning Bolt, Greaves, Helix, Berserker, Axe, while the Name sort's five leave Bolt out; "Sol" still leads with Sol Ring, an unranked name last in its tier. Red with `BestMatch` ordering by name.
- `card_sort_key.rs`: `all()` leaves `BestMatch` out; it travels as `"BestMatch"`.

## B
- `zwipe_ui::screens::deck_cards::quick_add_query` sends `CardSortKey::BestMatch`; the app's deck Cards takes it from the model, and the web's Quick add (`zwiper-web/.../deck/cards/quick_add.rs`) now builds its query with the same function instead of its own copy.
- The commander maybeboard's Quick add (`commander_maybeboard::search_query`) still sends Name.
- Tests: `quick_add_query_turns_synergy_off_and_sorts_by_best_match`; the zerver `quick_add_names` helper rebuilds the query with `BestMatch`, and the Quick add tests pass under it.
