# 454. On phone web, the featured chips touch Quick add

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-12 (/tmp/454-mvp-chips.jpg): phone web, a deck's Cards: the featured strip's MVP chips touch the top of the Quick add bar, with no gap between the strip and the controls. Give it the gap the app uses there.

## Fix
thinker:
- The app's strip keeps 1rem under it (`.deck-featured`); in the web's Cards layer the head, which ends with the featured cards, now keeps the same 1rem (`.web-cards-layer .deck-head`).

## Verification
- Phone web, a deck with MVPs, its Cards: a 1rem gap between the MVP chips and Quick add, as in the app.
