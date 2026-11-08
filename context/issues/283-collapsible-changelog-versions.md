# 283. Collapsible versions in every changelog

State: fixing | Owner: laptop | PR: - | Build seen: 1e03520ef

## Report
Scotty, 2026-11-07, feature request: every changelog's version panels collapse, with a caret to toggle each one, plus a "Collapse all" button on every changelog. Call sites: the app's changelog (About), the web app's changelog, zwipe.net's changelog, and the portfolio's Cairn changelog (portfolio repo). Prefer one shared component (orangui if the portfolio can use it). Everything starts expanded (Scotty); "Collapse all" collapses every version, then any can be reopened. The button can flip to "Expand all" when all are shut.
