# Versioning

Owner convention, decided 2026-08-11 (previously vibes), reworded 2026-10-23:

The goal is a balanced changelog. zite's Changelog page has one tab per minor line (1.13, 1.12, ...), so each tab should stay short and roughly as heavy as its neighbors.

- Minor bump (1.7 → 1.8): a real new capability (a new screen, flow or behavior), or the current minor line's tab is getting long and a fresh one reads better.
- Patch bump (1.8.0 → 1.8.1): fixes, polish, copy, and small additions to things that already exist (an extra detail on a row, a count on a chip). 1.13.1's link expiry on Profile's Pending row is the model.
- When in doubt, pick whichever keeps the tabs more balanced.
- Major stays reserved for a product-shape change (owner's call when it comes).

Why: release pages group by minor line, and 1.7.x accumulated seven feature trains before this was written down: heavy pages, and a patch digit that had stopped meaning anything.

History note: releases through 1.7.6 predate this rule (feature-carrying patches were normal). Do not reinterpret old numbers; the rule applies from 1.8.0 onward.

Mechanics stay unchanged: the version lives in the workspace `Cargo.toml` (`[workspace.package]`), bumps at cut time with the changelog move (`UPCOMING` → `RELEASES`), and iOS build numbers / Android versionCodes increment independently per upload (see the submission `history.md` files).
