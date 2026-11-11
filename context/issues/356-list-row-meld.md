# 356. A meld card's list row has no Meld toggle

State: fixing | Owner: scotland | PR: #PR | Build seen: release 9b2a9c475, web

## Report
scotland, 2026-11-11, found while checking #554: on the deck page's list, an open meld half (Bruna, the Fading Light) shows no Meld toggle, so `v` on it does nothing there. The Stacks details drawer and the card image have one. Scotty asked for the toggle on list rows, reusing the drawer's, with `v` toggling it through the shared key map (`shortcuts::card_key`).

## Cause
`CardDetails`, the open row's body and action bar, never used the card's meld toggle. Only `CardDetailsPanel` (the drawer) and `FlippableCardImage` did.

## Fix
scotland, #PR (components): the drawer's Meld button is one `MeldButton` component, shared by the drawer and `CardDetails`. On a meld half under a host `MeldSource`, the open row shows Meld in its action bar after Flip. With Meld on, the row's body shows the melded card, as the drawer does. `v` on the deck page presses it through `data-action="meld"` (#554). The app's rows use `CardDetails` too, so they get the toggle where the app provides a `MeldSource`. Failing-first tests in `components/tests/meld_row.rs`. Headless on scotland: the open Bruna row shows Meld; `v` turns it on (Brisela's rules text) and off again. Shot `target/shots-356/356-row-meld.png`.
