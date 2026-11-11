# 346. Charts animate in when they open

State: fixing | Owner: laptop | PR: - | Build seen: 918a1d477

## Report
Scotty, 2026-11-11, approved: every chart in Zwipe should animate in, both when its page opens (Playstyle) and when a folded section holding it expands (deck stats: Distributions, Mana, Draw odds; Activity's heatmap and charts). It should reuse orangui's chart entrances as the portfolio and Cairn use them, with no new animation style. It should respect reduced motion and play once per open or expand, not on every data refresh, in the app and on the web.

## Charts
- `BarChart` (components): rows, chip rows and vertical columns. Used by Activity (Totals, the role and tag charts), Playstyle (roles, tags) and Suggested commanders.
- `ManaCurve`, `DrawOdds`, `ManaFulfillment` and `BarChart` inside `DeckCharts`: the deck page's Distributions, Mana and Draw odds (app, web, and the shared deck page on zwipe.net).
- `ColorChart` and `ColorBars`: the deck's colors, Playstyle's Colors, Suggested commanders' colors and Home's color bars.
- `HeatGrid` (orangui): Activity's heatmap. Its cells already sweep in on mount (`heat-sweep`).

## Fix
- orangui#34 (merged, 96f46a4) moved the portfolio's bar entrance into the kit beside `chart-draw`: `.chart-bar` grows with `chart-grow`, `.chart-bar.chart-bar-up` with its vertical twin `chart-rise`, and `.chart-value` fades in with `chart-fade`. They use the portfolio's timing (0.6s, `cubic-bezier(0.33, 1, 0.68, 1)`) and go off under reduced motion.
- zwipe repins orangui, and every chart bar takes `.chart-bar` (columns take `.chart-bar-up`), staggered 45ms per row as the portfolio's fleet chart is.
- Replay on expand: a shut fold's body is `inert`. That holds for the web's `DeckFold` and section chrome, the app's `collapsible_section` and orangui's `Panel`. Under `[inert]` the entrance is off, so opening a fold starts it fresh, while a data refresh only restyles bars that are already there. Draw odds' rows remount on each turn step (a WebView workaround), so only the first render's rows grow in.
- The colored bars with mana pips (Colors, Mana cost fulfillment) grow their fill inside the track while the pip and count stay put, with the same timing and easing.
