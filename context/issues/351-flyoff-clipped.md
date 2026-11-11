# 351. The commander swipe's fly-off is cut off before the window's edge

State: fixing | Owner: scotland | PR: #PR | Build seen: live ce1cec30d, web, a wide window

## Report
Scotty, 2026-11-11, web, Playstyle's Suggested commanders swipe: the fly-off plays, but the flying card is cut off by an invisible box before it reaches the window's edge. Recording `target/scotty-shots/351-flyoff-clipped.mov`.

## Cause
The swipe body is a `.screen-content.card-swipe`, and the shared sheet (base.css) gives it `overflow-y: hidden`, so it clips on both axes. On a wide window swipe select's body is capped at `--web-content-max` and centered (`.web-swipe-select-body`, web.css), so the card is clipped at the body's left and right edges: x 144 to 1296 at 1440 wide, and further in on a wider window. Add and Remove are not affected: their body sits in `.web-scroll-screen`, whose rule sets it `overflow: visible`, so only the full-width screen clips them. Below 900px the body is the window's width.

## Fix
scotland, #PR (web.css): from 900px swipe select's body is `overflow: visible`, and its screen (`.swipe-select-screen.web-swipe-select`, fixed to the window under the nav) clips instead, so a skipped or picked card flies to the window's edge and the page never scrolls sideways. Failing-first test `a_skipped_or_picked_commander_flies_past_the_body_s_edge` (`cards/swipe_select.rs`). Headless, slowed to a tenth: at 1440 the card is cut at x 144 before and reaches the window's edge after; checked at 1920, 1440, 1000 and 390. Shots in `target/shots-351/`. The app's swipe body has no width cap, so it clips at the screen's edge; droid rechecks.
