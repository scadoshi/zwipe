# 459. The Decks list jitters as its Tags fold opens and closes

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: 4059d64f8

## Report
Scotty (/tmp/459-tags-jitter.mov): web Decks list, the Tags dropdown jitters as it opens and closes when the decks are laid out; with every deck group collapsed it's smooth. surfer's before (halo ~/surfer-shots/before-459/): Chrome macOS 1009 wide, 13 decks, All open, toggle Tags: on a bad toggle the list's first frame is already at the end and it swings ~30px the wrong way and eases back. Group open at least 4 of 12 real clicks and 5 of 12 scripted; every group collapsed 0 of 12.

## Cause
- The Tags chip sits in the Show row, which is a settle trigger (`"data-settle"`), so a Tags click starts the page settle's watch (`components/src/settle.rs`). The fold only grows or shrinks, but the watch, measuring the keyed deck rows while the fold's height transition moves them, slides them back with a FLIP: surfer's probe caught a 250ms WAAPI translate of -79.19px (the tag row's height) to 0 with the settle's `--ease-enter`, on all 13 rows, on every bad toggle and none of the good ones (the settle's own signature, `settle.rs` line 174; `row-enter` is a CSS animation and reads by name). With every group collapsed the scope holds no deck rows to move, so it's smooth.
- The settle already exempts folds through `NOT_TRIGGERS`, but only the deck page's fold-all button.

## Fix
thinker:
- `NOT_TRIGGERS` is `.deck-fold-all, .web-deck-tags-fold`, so the Tags fold never starts the settle; the tag chips inside it still do.
- Test `the_tags_fold_never_starts_the_settle` (zwiper-web, deck list): red before, green after.

## Verification
- surfer's before steps: 1009 wide, 13 decks, All open, toggle Tags 12 times: the list slides one way each time, no wrong-way swing, and the probe logs no WAAPI animations on the deck rows.
