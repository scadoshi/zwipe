# 399. The deck page's List is slow in Firefox: opening a row, folding, Art

State: fixing | Owner: scotland | PR: - | Build seen: live, web, Firefox, a 103-card deck

## Report
Scotty via composer, 2026-11-12: on app.zwipe.net's deck page in List view, toggling Art, opening a row and fold or unfold all are "super slow", worst on opening a row and on fold all, even after 374 (#585). The same actions on the deck's zwipe.net share page are instant. His browser is Firefox.

## Measured
Headless Firefox 160 and headless Chrome on scotland, optimized local builds (opt-level 3) of both sites, the same 101-row deck shared, 1422 wide, real mouse clicks. Each figure is the median of five runs of the Event Timing click duration, from the click to the next paint, in ms.

| | Firefox web | Firefox zwipe.net | Chrome web | Chrome zwipe.net |
|---|---|---|---|---|
| Art toggle | 480 | 72 | 384 | 136 |
| open or close a row | 448 | 24 | 192 | 96 |
| fold a group | 280 | 64 | 152 | 120 |
| fold all | 296 | 72 | 296 | 192 |

In both browsers the click handlers take under 40ms and the first DOM change lands at the same time on both sites. The extra time is all style and layout before the paint. Surfer's real Chrome numbers on halo agree: the same handlers, and 16 to 32ms more on the web in style, layout and paint. Folding a group in Stacks is fast on the web (48ms in Firefox), so the cost belongs to the List.

## Cause
The List's two columns from 1400px (350) are a CSS multi-column, `column-count: 2` on `.deck-bands`, which Firefox balances again over the whole list on every change inside it. Bisected in Firefox by removing rules from the live page:
- **All 69 `:has()` rules removed:** a row opens in 24ms.
- **Only the three List column rules removed:** 32ms. These are the `.deck-bands:not(:has(> .deck-stacks))` rules, and removing them also removes the columns.
- **The same columns on a plain selector, without `:has()`:** 928ms. So the cost is the multi-column, not the `:has()`. `contain: layout`, `break-inside: avoid` on every row and `display: flow-root` on the groups did not help (488 to 728ms).
- **Every other `:has()` group made no measurable difference:** the `html` and `body` ones, the scroll screen's, the hover lift, `row-enter`'s and the input wraps.
- **Not the cause:**
  - The settle script attaches nothing at rest and reads layout only after a click in a Boards or Group by chip row.
  - The dock's `footer-rise` is a scroll-driven animation, which costs nothing at idle and runs on zwipe.net too.

## Fix
scotland:
- **Two grid columns:** the List draws its groups into two plain columns, a grid of two from 1400px. The groups keep their order, the first ones on the left, split where the columns hold the most even share of rows (`zwipe_ui::deck_cards::columns::split_at`).
- **Below 1400px:** the column wrappers are `display: contents`, so the groups flow as one column as before. On the test deck the split puts the same groups in each column as the multi-column did.
- **Folding:** a fold no longer re-balances the columns, so groups stay in their column while folded.

After, same builds and method:

| | Firefox web | Chrome web |
|---|---|---|
| Art toggle | 56 (was 480) | 256 (was 384) |
| open or close a row | 32 (was 448) | 184 (was 192) |
| fold a group | 72 (was 280) | 144 (was 152) |
| fold all | 80 (was 296) | 264 (was 296) |
