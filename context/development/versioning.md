# Versioning

Owner convention, decided 2026-08-11 (previously vibes):

- **Minor bump (1.7 → 1.8): the release carries ANY feature.** New capability,
  new screen element, new behavior a user can see — however small.
- **Patch bump (1.8.0 → 1.8.1): fixes, UI tweaks, copy, polish only.** A patch
  number now tells users "nothing new, just better" (the 1.7.4 crash-fix train
  is the model).
- Major stays reserved for a product-shape change (owner's call when it comes).

Why: release pages group by minor line, and 1.7.x accumulated seven feature
trains before this was written down — heavy pages, and a patch digit that had
stopped meaning anything.

History note: releases through 1.7.6 predate this rule (feature-carrying
patches were normal). Do not reinterpret old numbers; the rule applies from
1.8.0 onward.

Mechanics stay unchanged: the version lives in the workspace `Cargo.toml`
(`[workspace.package]`), bumps at cut time with the changelog move
(`UPCOMING` → `RELEASES`), and iOS build numbers / Android versionCodes
increment independently per upload (see the submission `history.md` files).
