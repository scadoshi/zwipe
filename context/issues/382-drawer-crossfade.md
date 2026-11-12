# 382. The Stacks drawer snaps to another card

State: fixing | Owner: laptop | PR: #606 | Build seen: audit build 9680dcacb

## Report
Ease audit, row 6 (`~/ease-audit/README.md`), moved from thinker via the composer: clicking another card while the Stacks drawer is open replaces its contents in one frame, with no crossfade, and the title shifts.

## Fix
laptop, #606, web: `cards::drawer_swap`, installed by `CardDetailsSheet`.
- **Copying the old card.** A click on a stack card, or a key, while a card details drawer is open copies the drawer's title bar and content as they show.
- **Fading.** If the drawer then shows another card, a mutation observer lays the copies over the drawer in the same task, before the browser paints the change. The copies fade out over 200ms while the new card fades in.
- A drawer that kept its card drops the copies unseen. Nothing runs under reduced motion.

## Checked (headless Chrome, 1422)
A per-frame probe sampled the drawer while a click swapped Beetleback Chief for Goblin Chieftain. Each entry is ghost count / ghost opacity / live opacity / title:
- On the first frame with the new title, the copies cover it fully (2 / 1.00 / 0.00), so the new card never shows unfaded.
- They then trade over about 12 frames (0.93 / 0.07, 0.59 / 0.41 … 0.00 / 1.00).
- Under reduced motion the swap is instant, as before.

## Verification
-
