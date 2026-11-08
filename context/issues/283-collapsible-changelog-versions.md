# 283. Collapsible versions in every changelog

State: fixing | Owner: laptop | PR: - | Build seen: 1e03520ef

## Report
Scotty, 2026-11-07, feature request: every changelog's version panels collapse, with a caret to toggle each one, plus a "Collapse all" button on every changelog. Call sites: the app's changelog (About), the web app's changelog, zwipe.net's changelog, and the portfolio's Cairn changelog (portfolio repo). Prefer one shared component (orangui if the portfolio can use it). Everything starts expanded (Scotty); "Collapse all" collapses every version, then any can be reopened. The button can flip to "Expand all" when all are shut.

## Fix
One orangui piece, `orangui::collapse` (orangui#26, merged): `use_collapse_set` holds which sections are shut (everything starts open), `CollapseCaret` turns ▸ down while open, `CollapseBody` eases shut and goes inert, and `CollapseAll` reads "Collapse all" until every section is shut, then "Expand all". Tests: `Shut` unit tests (toggle, collapse all, reopen one, empty list) and two HTML goldens.

zwipe (`nits/laptop-283`, #417): `components::ChangelogReleases`, which draws the app's About changelog, the web app's About changelog and zwipe.net's changelog, gives each version a caret and wraps its notes, with Collapse all at the right over the shown releases. The orangui pin moves to orangui#26; the changelog goldens are re-blessed. Shots on `nits/laptop-283-shots` (`shots/283-*.png`).

portfolio (scadoshi/portfolio#20): Cairn's changelog cards take the same orangui pieces, with the orangui pin moved to main.
