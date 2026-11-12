# 403. Home's Colors panel stops short of Decks beside it

State: fixing | Owner: thinker | PR: #TBD | Build seen: release e556ab671, local

## Report
Scotty, through the composer, 2026-11-12, after renders at 1634 and 1400: on app.zwipe.net's Home, Colors (227px) sits beside Decks (328px) and ends well above it. He picked the stretched render. Colors should run to Decks' height, with its bars at the top and the rest empty panel, only where the two sit side by side. Stacked, each keeps its natural height.

## Cause
The shared `.web-columns` grid aligns its columns to the start, so each panel keeps its own height.

## Fix
thinker: in Home's 900px-and-up block, `.web-home-page > .web-columns` stretches its columns and `.home-colors` takes `flex: 1 1 auto` in its column. The rule is scoped to Home, so About, Playstyle and Activity keep the shared `align-items: start`. Below 900px the columns are `display: contents` and the rule does not apply.

## Verification
-
