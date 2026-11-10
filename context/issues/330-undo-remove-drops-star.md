# 330. Undo of a remove brings the card back without its star

State: fixing | Owner: thinker | PR: - | Build seen: live 1.18.16

## Report
Scotty, 2026-11-09: on a deck's card list, star a card, Remove it, then Undo the remove. The card comes back fresh, without its star. Undo must restore the card exactly as it was.

## Diagnosis
thinker, 2026-11-09. Undo re-creates the removed row through the add call (`HttpCreateDeckCard`: printing, quantity, board), then takes the server's fresh `DeckCard`, where `mvp_at` is empty. The star is the only per-row field lost; tags are the card's, not the row's. The same path is in three places: the app's shared model (`zwipe-ui` deck_cards `re_add` and its create reply), web's editor and web's remove screen.

## Fix
Scotty picked the exact restore, in two PRs.
- PR 1, server (deploys first): `HttpCreateDeckCard` gains an optional `mvp_at`. The server inserts with it only on the mainboard, only when it is not in the future, and only while the deck has fewer than three stars, counted under the deck lock. Otherwise it creates the card unstarred and does not fail. A future time is ignored, not refused. Tests first in `zerver/tests/deck_cards.rs`: a past `mvp_at` is kept with its time, a future one is not, a full podium and a maybeboard create come in unstarred, and a create without the field is unchanged. In zwipe-core, an unstarred create serializes without the field.
- PR 2, clients (after the deploy): the undo of a remove sends the removed row's `mvp_at` in the app, web's editor and web's remove screen, with the failing test first: remove then undo keeps the star, quantity, board and printing.
