# 374. The deck page's List is slow to open a row, fold or toggle Art

State: fixing | Owner: scotland | PR: #PR | Build seen: live, web, a full deck

## Report
Scotty via composer, 2026-11-12, screen recording: on the web deck page in List view, toggling Art, opening a row and collapse or expand all are visibly slow on a full deck.

## Measured
Headless Chrome on scotland, a 100-row deck at 1422 wide, release with opt-level 3 (a release-profile build refuses an http backend), medians of six runs. "Click to DOM" is the click to the first DOM change; "click to frame" is that plus the next painted frame.

| | click to DOM | click to frame |
|---|---|---|
| Art toggle | 20ms | 40ms |
| open one row | 32ms | 47ms |
| collapse or expand all | 32ms | 64ms |

From a Performance trace of the same interactions, in main-thread ms:

| | script | style | layout | paint |
|---|---|---|---|---|
| open one row | 39 | 9 | 6 | 92 |
| collapse all | 56 | 26 | 7 | 95 |
| Art on | 40 | 43 | 24 | 148 |

Opening a row changes 5 DOM nodes, yet costs as much script as folding every group.

## Cause
- **Script (Dioxus):** every List row came from `row_view`, a plain function called in the deck page's own render, reading `selected()` and the faces map. Opening one row, or turning one face, re-rendered the whole page and diffed all 100 rows. `CardRow` also read the shared `expanded_card` directly, so all 100 rows subscribed to it.
- **Layout:** small (6 to 24ms). The two-column `column-count` list is not the cost.
- **Image decode:** off the main thread (0ms on main).
- **Paint:** mostly the expand and fade transitions painting each frame, inflated here by software raster. CSS containment (`contain` on rows, `content-visibility: auto` on the bands) changed nothing measurable, so it is not part of the fix.

## Fix
scotland, #PR:
- `row_view` draws each row as its own keyed component, `DeckRowItem`, with plain props. It reads whether it is the open row and the face it shows through memos of its own, so the page and the other rows stay put when one row changes. Folding leaves unchanged rows alone. `Editor` derives `PartialEq` (its fields compare by identity) so it can be a prop.
- `CardRow`'s open state is a memo of `expanded_card`, so the rows sharing it render again only when their own open state flips.

After, same build and method:

| | click to DOM | click to frame |
|---|---|---|
| open one row | 7.5ms (was 32) | 23ms (was 47) |
| collapse or expand all | 16ms (was 32) | 48ms (was 64) |
| Art toggle | 20ms (unchanged) | 40ms (unchanged) |

Script for opening a row drops from 39 to 16ms. Art is unchanged: every row's art changes, so every row renders; it is no longer the slowest of the three. Phase 2 of 370 moves these rows into components as they are now.
