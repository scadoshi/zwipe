# 396. app.zwipe.net Home puts its three panels in one row from 1920

State: fixing | Owner: thinker | PR: #602 | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12, with a screenshot of Home at 1400: the row under the hero is two columns, Decks and Colors side by side sharing its width, and Featured flavor always takes its own full-width row below them, at every desktop width. The phone stays one column.

## Cause
#561 laid Decks, Colors and Featured flavor in one three-column row from 1920 (`@media (min-width: 120rem)` in `web.css`).

## Fix
thinker: that rule is gone, so Home keeps its two-column row with the flavor card under it at every wide width.

## Verification
-
