# 397. Playstyle, Activity and the Commander maybeboard lists are List only

State: fixing | Owner: laptop | PR: - | Build seen: release after #591

## Report
Scotty, via the composer, 2026-11-11: remove Stacks from the one-off lists: Playstyle's Suggested commanders and Staples, Activity's Most skipped, and the Commander maybeboard. They are List only, with no View switch, no stack code paths, and no saved view setting (any stored value is ignored). The card keys stay on the rows. The deck page keeps its List | Stacks.

## Fix
laptop, web:
- The four lists render rows only: no View row, no stack, no stack drawer. Art no longer grays out on these screens.
- Removed the code only they used: `CardsStack`, `StackDrawer`, `use_stack_details`, and the per-list `CardsViewPlace` keys. `CardsViewStore` is the deck page's one key again (`zwiper-web.deck-cards-view`). The old per-list keys are never read.
- The card keys stay: `p`, `i`, `v`, `d` and Shift+H on each screen's rows, and the maybeboard's `/` and `f` still pass through.
- Changelog: the unreleased Stacks lines become key-only lines.

Headless check at 1422: on each screen, no View switch, no stack, and `d` over a row opens it. Shots: `target/397-shots/` on the laptop.

## Verification
-
