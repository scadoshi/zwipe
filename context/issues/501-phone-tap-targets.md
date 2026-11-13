# 501. On a phone, a card row's actions and the changelog's carets are small tap targets

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Surfer's walkthrough at 390, triaged by Scotty (composer, 2026-11-13): row actions (Image, Printings, Create deck, Maybeboard) are about 22px tall and the changelog's collapse buttons 20x20; raise them to a comfortable touch size without changing the wide layout.

## Before
Headless at 390: every card action (`.card-action-btn`: Image, Printings, Create deck, Maybeboard, Remove on Playstyle; Image, Printings, the stepper, Star, To maybeboard, To sideboard on a deck's List) stands 22px tall. About's release carets measure 20x20, but each sits in its release's head, which toggles from anywhere on it: 356x56, and a tap 70% across it folds the release (1 of 1).

## Cause
`components/assets/magic.css`, `.card-action-btn`: 0.2rem of padding around 0.85rem text at line-height 1, at every width.

## Fix
scotland:
- Below 900px a card action is at least 2rem high and reaches 0.375rem past its edges (`::before`, the kit's `banner-dismiss` reach), about 44px in all; `.card-action-row` spaces them 0.75rem so neighboring reaches don't overlap. A wide window is unchanged.
- The carets are left as they are: their tap target is the release's whole head.

## Verification
- Headless at 390: every card action 32px high with a 44px reach on Playstyle and a deck's List; no two visible reaches overlap (Playstyle's five, wrapped). At 1440 they stay 22px with no reach.
- Surfer's after on 869fced20: Playstyle passes; a deck's List with a card open fails: its two action rows (Image, Printings, the stepper and Star; then To maybeboard and To sideboard) stand 8px apart, so their reaches overlap (a tap 3px under Printings hits To maybeboard), and the bottom row's reach is clipped by the open row's `overflow: hidden`. Before, headless 390: gap 8, five overlapping pairs, the reach clipped at 488 of 490. Cause: `.card-action-row` keeps its 0.5rem top margin on a phone, less than two reaches. Fix: on a phone a row stands 0.75rem below the one above, and the last keeps 0.375rem under it. After: gap 12, no overlapping pairs, nothing clipped; Playstyle still none.
