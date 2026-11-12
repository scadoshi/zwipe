# 389. A deck's Edit, Clone and Share sit in the Profile header on a phone

State: fixing | Owner: thinker | PR: #593 | Build seen: production, app and web

## Report
Composer for Scotty, 2026-11-12: on the deck profile screen at phone size (the app, and the web below its breakpoint, where the deck shows as Profile then Cards), move the Profile header's Edit, Clone and Share into the action dock, so it reads Edit, Clone, Share, Buy, a separator, then the Cards button: the same icons and order as the wide web dock. Remove them from the header so each exists once.

## Fix
thinker:
- `zwipe_ui::screens::deck_view::dock` gives the dock's order (Edit, Clone, Share, the cart when there is a store link, the divider, Cards), and both clients draw their dock from it with the dock's own icon buttons and `action-dock-divider`;
- the Profile header loses its buttons in both clients (`ProfileActions` and `DeckProfileSection`'s `actions` go, with the `.deck-profile-actions` rules and the web's loading ghost icons);
- the first-deck hint's phone copy says the pencil is in the dock.

The wide web page is unchanged.

## Verification
-
