# 482. Quick add leaves a very popular card behind less played names that start with the query

State: fixing | Owner: thinker | Type: feature | PR: - (pushed to release) | Build seen: 045bd5c38

## Report
Scotty, via the composer, after 473: very popular cards may outrank "starts with" in Quick add's best match only, so "bolt" offers Lightning Bolt, "ring" Sol Ring and "dragon" The Ur-Dragon. Search's Name sort stays as it is. In-deck cards stay hidden.

## Before
The ordering is the server's `ORDER BY`, so the before is the query's answer on a pool of real names at their real EDHREC ranks (Scryfall), The Ur-Dragon the most-helmed commander: "bolt" gives Bolt Bend, Bolt Hound, Boltwave, Boltbender, then Lightning Bolt; "ring" leaves Sol Ring out of the five (it is in the word-start tier, behind Ring of Three Wishes and the "ring..." names); "dragon" leaves The Ur-Dragon out.

## Cause
- `CardSortKey::BestMatch` (473) ranks popularity only inside a tier, and a word match ("% bolt%") sits below every name that starts with the query (`zerver/src/lib/outbound/sqlx/card/mod.rs`, `push_tiers`).
- The Ur-Dragon is only 2702 by EDHREC rank (a top commander, not a top card), and the name fold drops the hyphen without a space, so "The Ur-Dragon" holds no word that starts with "dragon".

## Fix (rule approved by Scotty, via the composer)
1. The exact name first, as before.
2. A card's popularity is its EDHREC rank, or, for one of the 100 most-helmed commanders in `commander_popularity`, its place among them when smaller. It is read through scalar subqueries, not a join.
3. A card with popularity 300 or better and a word that starts with the query (hyphens break words for this check) joins the first-word tier. A match inside a word never lifts (Smothering Tithe for "ring", Isolated Chapel for "sol").
4. Each tier orders by that popularity, unranked last, then by name.

Search's Name sort keeps its tiers without the lift and orders by name.

## Tests
- `a_very_popular_word_match_joins_the_first_word_tier`: bolt, ring, dragon. Red before; red with the lift allowed inside a word (Smothering Tithe second for "ring"); red without the hyphen break (no Ur-Dragon).
- `the_closest_names_still_lead_where_nothing_popular_is_further_down`: sol (Sol Ring, then Solemn Simulacrum, lifted at 44), mountain (Mountain first), lightning (Lightning Greaves, Lightning Bolt).
- `best_match_ranks_each_name_tier_by_popularity` (473) now uses real ranks; its made-up ones (Lightningsnare 15, Chain Lightning 300, Abby 5) were very popular under this rule.

## Not checked
- Whether prod's `commander_popularity` holds The Ur-Dragon in its top 100; without it "dragon" leads with Dragonskull Summit.
- The query's cost on prod data: the commander subqueries run per matched row; a lookup by primary key, then a count only for a commander.
