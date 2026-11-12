# 388. Playstyle and Activity stacks sit in panels, and share one View setting

State: fixing | Owner: laptop | PR: #591 | Build seen: local, web

## Report
Scotty, via the composer, 2026-11-11, follow-up to #574: in Stacks, Suggested commanders, Staples and Activity's Most skipped show their stack inside the section's panel, unlike the deck page, where the cards float on the page under a card-width group header chip. One View setting drives every section. The Commander maybeboard has no List | Stacks switch.

## Repro
1. On the web at 1422, open Playstyle and pick Stacks.
2. Both lists stack inside their panels, and Staples follows the same switch.
3. Open the Commander maybeboard: there is no View row.

## Fix
laptop, #591, web:
- Each list keeps its own choice in browser storage (`CardsViewPlace`): Suggested commanders, Staples, Most skipped and the Commander maybeboard each start on List; the deck page keeps its key.
- In Stacks a list renders as `CardsStack`: the deck page's `DeckGroupSection` header (title, count, the section's buttons) over a `CardStack`, in a `.deck-stacks` grid, with no panel.
- Staples has its own View row beside its Type and Color filters. Art grays out only while both Playstyle lists are stacked.
- The Commander maybeboard has a View row after Art, one stack with the details drawer (Create deck, Remove, Image, Printings) and the card keys. Its own keys (`/`, `f`) still work, since the card keys pass every other key on to the screen.
- Tests: each place keeps its own choice and starts on List; the deck page's key is unchanged.

## Verification
-
