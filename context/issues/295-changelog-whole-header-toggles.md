# 295. Changelog versions only toggle from the caret

State: approved | Owner: scotland | PR: - | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot of a collapsed changelog, 1.18.12 and 1.18.11 Upcoming): to expand a version you have to click the small caret. The whole version panel header (caret, version number, tags) should be clickable and toggle it. Every call site from 283: the app's About, the web app's changelog, zwipe.net's changelog, and the portfolio's Cairn changelog.

Scotty, 2026-11-08: also swap the "Expand all" / "Collapse all" words for the same double-arrow icon the app and web app use for that toggle (aria-label keeps the words), on every changelog.
Placement (Scotty): the icon sits in line with the Changelog title, at the right end of the title row, not floating above the first version.
