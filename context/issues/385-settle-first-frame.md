# 385. The settle paints the final layout for one frame first

State: fixing | Owner: laptop | PR: - | Build seen: local, web

## Report
Surfer, via the composer, 2026-11-12: #578's Maybe toggle in Stacks paints the final layout for one frame (about 13ms) before the slide starts. The first painted frame after a click should already be the starting state, in List and Stacks, for Boards and Group by.

## Cause
The settle script's `MutationObserver` saw the bands change and scheduled the moves with `requestAnimationFrame`. Dioxus applies its edits inside a frame, after that frame's animation callbacks, so the browser painted the new layout once before the next frame started the slides and fades.

## Fix
laptop: the observer's callback runs the moves itself. It runs in the task that changed the bands, before the browser paints, so every slide's start and every new card's opacity 0 are in place on the first painted frame.

Measured per frame (rAF samples of a tracked card and the count of the script's own animations), before and after:
- **List, Side on:** before, frame 2 had the new rows at their final position (755) with no animation, and the slide began on frame 3. After, frame 2 has the old position (559) with 3 animations running.
- **Stacks, Side on:** before, the new cards appeared on frame 3 with none of the 8 fades running until frame 4. After, they start on frame 3.
- **Group by, List and Stacks:** the animations now start on the frame the DOM changes.

## Verification
-
