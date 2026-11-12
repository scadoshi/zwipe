# 368. A staple's deck count names no decks

State: fixing | Owner: laptop | PR: - | Build seen: production, web

## Report
Composer for Scotty, 2026-11-11: on Playstyle's Staples, the expanded row's "4 of 11 decks" chip should be a toggle chip (the Other tags pattern) revealing one chip per deck the card is in, each opening that deck.

## Cause
`HttpStaple` carried only the count. The server's `rank_staples` built each staple's set of deck ids and dropped it.

## Fix
laptop, a wire change, additive (see `development/api_evolution.md`):
- **zwipe-core:** `HttpStaple.deck_ids: Vec<Uuid>`, most recently updated deck first, `#[serde(default)]`. Compat tests:
  - a payload without the field reads with no deck ids;
  - a payload with it reads into the shape before it.
  - `HttpStaple` keeps accepting unknown fields.
- **zerver:** `rank_staples` keeps each staple's deck ids in input order (the mainboard query is `ORDER BY d.updated_at DESC`), once per deck. They flow through `Staple` to the response. The query is unchanged, so `.sqlx` is too.
- **zwipe-ui:**
  - `playstyle::StapleDeck` and `named_staple_decks` name the ids from the client's deck list, in the server's order, skipping an id the list doesn't hold.
  - The app's Playstyle model keeps the deck list (`DeckProfilesLoaded { decks }`), and `OpenDeck` navigates to `Destination::Deck`.
- **components:** `StapleDecksChip`:
  - with decks, a toggle chip easing open one chip per deck;
  - with none (an old server, or no names), the plain count chip as before.
- **Web and app:** the staple row's footer is the chip. A deck chip opens that deck's page.

## Verification
-
