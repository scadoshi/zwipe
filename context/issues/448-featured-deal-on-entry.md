# 448. The featured strip no longer deals in when the Cards open

State: fixing | Owner: thinker | Type: feature | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-12: on the phone, the featured strip (command zone and MVPs) should deal in again each time the Cards layer opens, even with the deck already loaded. halo's 7a51ede04 removed the deal to stop the page jumping; keep that fix: the strip holds its full height from the first frame and the cards deal into that reserved space, so nothing below moves. Phone app and phone web the same. scotland does the layer's grid tiles (same item) and the card details screen (449).

## Fix
thinker:
- App: `FeaturedCards` mounts with each Cards layer, and its cards deal (`.deck-featured.dealing`: a 0.5rem drop with a slight tilt and a fade, 0.12s apart, opacity and transform only) into the row's held height, then the class goes so a card starred later just joins.
- Web: the head's featured cards deal once they're ready and again each time the phone's Cards layer opens over them (`deck-head-deal ready`, a newer deal outdating the one before), into the head's held row.

## Verification
- Phone app and web at 390: open a deck's Cards, close, open again: the strip's cards deal in each time, and the controls under the strip don't move.
