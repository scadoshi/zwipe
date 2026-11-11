# 350. The web deck page leaves a wide window's sides empty

State: fixing | Owner: scotland | PR: #PR | Build seen: local, web, a modest desktop monitor

## Report
Scotty, 2026-11-11, web: on a desktop monitor the deck page sits in a narrow centered column with empty grid on both sides, in Stacks (five stacks across, room for more) and in List (one column of groups). Shots `target/scotty-shots/wide-stacks.webp` and `target/scotty-shots/wide-list.webp`. Asked for: Stacks take most of the window's width, keeping the side gutters and the stacks' card width; List shows its groups in two columns on a wide screen, never more; the controls and the stat panels stay sensible.

## Cause
The deck page is a `.web-page`, held to `--web-page-max` (74rem) from 900px like every other page, and its list's groups (`.deck-bands`) are one flex column.

## Fix
scotland, #PR (web.css):
- From 900px the deck page drops the page cap and runs the window's width inside the 1.5rem gutter. Head, controls, bands, stats and foot all take that width. The stacks grid (`auto-fill`, 12.5rem minimum) fits more columns, each about 215px as before: 6 at 1440, 8 at 1920 (5 before).
- From 1400px, the breakpoint where the Decks list pairs its rows, the List view's groups flow down two CSS columns, each group whole in one column (`break-inside: avoid`). Groups are never split and there are never more than two columns. Stacks are untouched (`.deck-bands:not(:has(> .deck-stacks))`).

Failing-first test `the_deck_page_runs_the_window_s_width_and_pairs_its_groups_from_1400` (`chrome/media.rs`). Headless shots in `target/shots-350/`: `{list,stacks}-{1000,1300,1440,1920}.png`, and the stat panels open at `stats-{1440,1920}.png`.
