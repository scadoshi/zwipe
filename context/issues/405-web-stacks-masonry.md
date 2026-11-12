# 405. The web deck page's Stacks hold each row to its tallest stack

State: fixing | Owner: halo | PR: - | Build seen: cdd789b1c

## Report
From the composer: bring #615's phone masonry to the web deck page's Stacks (the pointer grid). Each row of stacks takes the height of its tallest stack, so a 1-card stack next to a 50-card one leaves a gap. Each column should run on its own, every stack right under the one above it in its column, for any column count the width allows. Folds reweigh and slide through the shared settle. Hover reveals keep today's overlap and don't count toward height. No CSS multi-column (399's Firefox slowdown): flex columns. It lives in the shared `DeckStacks`, so zwipe.net's share page gets it with laptop's phase 4.

## Fix
halo:
- **zwipe-ui `deck_cards::columns`:** `place(heights, columns)` puts each stack under the shortest of any number of columns, the leftmost on a tie, and the phone's `masonry` is its two-column case. `column_count(width)` gives as many 12.5rem columns with 1.25rem between them as the width fits, as the grid's `auto-fill` did, never fewer than one. `stack_height` weighs a stack as on the phone; a hover reveal is a transform and counts for nothing.
- **components `DeckStacks`, pointer mode:** it measures the content box of the element it sits in with a `ResizeObserver`, in hundredths of a rem, and lays the stacks in that many flex `.deck-stacks-column`s. Until the first measurement it draws today's grid, so a first paint, and zwipe.net's static page, keep a sound layout. The last width is kept, so stacks mounted again lay out their columns from the first render. A header fold starts the settle (`settle::watch`) on both the web and the phone, so stacks that change columns slide there.
- **`web.css`:** three small rules for `.deck-stacks-columns`, with the grid's gaps.

## Checked
- Headless Chrome, the web's own sheets, a 28-card Creatures beside short groups. At 1400 (1336px of stacks, six columns), today's second row starts below Creatures, while the columns put Battles under Planeswalkers, Sideboard under Instants, Maybeboard under Sorceries and Lands under Enchantments. At 1920 (1856px, eight columns), the same.
- Headless Firefox at 1400: the same six columns in the same order.

## Verification
-
