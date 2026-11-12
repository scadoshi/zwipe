# 382. The Stacks drawer swaps cards in one frame

State: fixing | Owner: thinker | PR: - | Build seen: 9680dcacb, web (surfer's ease audit, ~/ease-audit/README.md, row C)

## Report
Surfer's ease audit, approved by Scotty, 2026-11-12: clicking another card while the Stacks drawer is open replaces its contents in one frame, with no crossfade, and the title shifts.

## Cause
The drawer (`CardDetailsSheet`) stays mounted and redraws its panel for the new card in place.

## Fix
thinker:
- A swap to another card while the drawer stays open crossfades: the card drawn before stays as a layer, inert and hidden from assistive tech, fading out over the new one. Both sit in one grid cell, on the kit's `--enter-duration` and `--ease-enter`.
- The incoming layer is keyed by the card, so its fade runs only on a swap. The title, keyed in `DrawerSheet`, eases in with it.
- A face or meld change, or a drawer opening on a card, swaps in place.
- Reduced motion hides the outgoing layer and skips the fades.

Headless at 1422, sampled every frame from the click: the outgoing card fades 0.54 → 0 while the new card and the title rise 0.46 → 1 together, done by about 240ms. Opening from closed onto another card shows no outgoing layer. Under reduced motion the old card is hidden at once.
