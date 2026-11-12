# 373. Board and Group by changes snap the deck page's cards

State: fixing | Owner: laptop | PR: #578 | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12: on the deck page, toggling the Boards chips (Main, Maybe, Side) makes cards vanish or appear instantly, and changing Group by snaps to the new grouping. Both should ease:
- **List:** rows and whole group bands fade and collapse out, and fade and expand in.
- **Stacks:** a hidden board's cards leave and the stack closes up, and they come back the same way.
- **Group by:** bands and rows settle into their new places, and stacks move smoothly.
- **Timing:** the shared `--ease-enter` over about 250ms.
- **Limits:** reduced motion skips it, nothing jumps once it settles, and filtering keeps its behavior.

## Cause
The bands re-render from the new listing in one step, with no record of where anything was.

## Fix
laptop, #578, one code path for both, `deck/view/settle.rs`: a script installed once on the deck page.
- **Before the change:** a capture-phase click on a chip row marked `data-settle` (Boards and Group by) records where every card sits (list rows and stack cards by `data-row`) and every group (by a new `data-fold` on `DeckFold` and on each stack).
- **After the bands re-render** (a `MutationObserver` on `.deck-bands`, then a frame):
  - a card or group still shown slides from its old place to its new one, on `translate`, so it composes with the stacks' own transforms; a card's slide is relative to its group's, so the two don't add up;
  - a new one fades in;
  - one that left fades out where it was, as a copy laid over the page;
  - a group that left takes its cards with it.
- **Timing:** `--ease-enter` over 250ms. Under `prefers-reduced-motion` nothing is recorded or moved.
- **Not triggers:** filtering, searching, folding and the View switch, which keep their behavior.

## Verification
-
