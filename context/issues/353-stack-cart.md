# 353. A stack has no way to buy its cards

State: fixing | Owner: thinker | PR: - | Build seen: local, web

## Report
Composer for Scotty, 2026-11-11, web: on the deck page's Stacks view, each stack header should carry a cart icon button next to its price tag (icon only, no "Buy" word), reusing the existing icon button and tag components. Clicking it opens a dialog with two choices, Card Kingdom and TCGplayer, which send that stack's cards to the store. It must fit the header at phone width. The phone app has no Stacks screen, so there is nothing to add there.

## Cause
Stacks shipped with a price per stack and no buy. The deck page had no store dialog either: Budget lists the two retailer links in a row, and only the phone app opens a Buy dialog (the featured flavor's).

## Fix
thinker: the app's Buy dialog becomes a shared `StoreDialog` (components), and a `CartButton` (the controls' `btn-icon` button with a cart `DockIcon`) sits in each stack's header after its price. The links come from zwipe-ui's `buy`, as the deck's do, for the stack's cards with their copies; tokens stay out. Issue 354 reuses the dialog for every other buy spot.
