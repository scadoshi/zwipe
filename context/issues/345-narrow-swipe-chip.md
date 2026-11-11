# 345. Add's last-action chip is missing below the full nav

State: fixing | Owner: scotland | PR: #537 | Build seen: live, web, about 1000px wide

## Report
Scotty, 2026-11-11, web: on Add at a narrower window (the nav folded), the chip at the top left that names the last swipe ("Added", "Skipped" and the like), shown on a wide window, does not show. Screenshot `target/scotty-shots/345-narrow-no-action-chip.webp` (Add, about 1000px, nav folded, no chip). Check Remove and the commander swipe too.

## Repro
1. Web, Add or Remove, a window from 900px to 1109px wide (the nav folded).
2. Swipe a card: no chip at the top left.

## Cause
The swipe screen's chips (`SwipeScreenChips`, `.web-screen-chips`) take the wide layout's placement from 900px: measured from the window's top (the screen content is static there), 1rem down. From 1110px the full nav sits above that box, so the chip lands under it. Up to 1109px the nav is folded and lies over the window's top instead, so the chip, 16px down, sits under the folded nav's bar and is hidden. Below 900px (phones) the chip shows under the title bar. The commander swipe (swipe select) starts under the nav from 900px, so its chip shows.

## Fix
scotland, #537: from 900px to 1109px, Add's and Remove's chips sit at the folded nav's height (`--web-nav-h`) plus 0.5rem at the left, where the phone layout puts them, and the title tag stays hidden there as on a phone, since the folded nav names the screen (web.css). Failing-first test `the_swipe_chips_clear_the_folded_nav` (`chrome/media.rs`). Headless, after a swipe: the chip at (16, 52) at 390, 1000 and 1100 on Add and Remove; at 1260 unchanged (73, 87).
