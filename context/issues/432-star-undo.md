# 432. Undo on the deck page skips starring and unstarring a card

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-12: starring or unstarring a card on the deck page (the Star button and Shift+S, web and app, owner page) isn't an undoable action, so Undo skips it. Add it to the same undo stack with the usual toast, restoring the previous star state and respecting the 3-MVP limit.

## Fix
thinker:
- zwipe-ui `UndoAction::MvpToggled { card_id, card_name, before }`, recorded once the server takes a star or an unstar (a refused one records nothing). Undo sets the star back to `before` and patches it, without recording, toasting "Unstarred <card>" or "Starred <card>".
- An undo the deck moved past (the card gone, or already back as `before` had it) says Already changed. Re-starring when three main board cards are starred toasts "This deck already has 3 MVPs" and sends nothing (`deck_cards::MAX_MVPS`, `mvp_count`).
- The app's deck cards view goes through the shared model, with the app's undo log carrying the new variant. The web editor's star goes through one `set_star` the toggle and the undo share.
- Shift+S runs the same toggle as the button, so it records too.

## Verification
- `cargo test -p zwipe-ui screens::deck_cards` (records on success only, undo unstars and restars, the cap, stale entries).
- Web and app, owner deck page: star a card, press Undo (or Ctrl+Z on the web): the star goes and "Unstarred <card>" shows. Unstar a card, Undo: it comes back. Star three, unstar one, star another, Undo twice: the second says the deck already has 3 MVPs.
