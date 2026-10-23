# Versioning

Owner convention, decided 2026-08-11, made plain 2026-10-23. Version numbers should read the way most people read them: the middle number goes up when there is something new, the last number when there are only fixes.

- Minor bump (1.13.0 → 1.14.0): the release adds anything a user can see, however small. A new screen and an extra detail on an existing row both count.
- Patch bump (1.14.0 → 1.14.1): the release only fixes or polishes things that already exist (bugs, copy, spacing, color). Nothing new to point at.
- Major bump: reserved for a change to what the product is. That is the owner's call when it comes.

If a release has one small feature and ten fixes, it is a minor. The question is only whether there is anything new.

The changelog no longer shapes the numbers. zite's and the app's Changelog group releases by the month they shipped, so a busy minor line or a long run of patches makes no difference to how the page reads.

History note: numbers through 1.13.0 were chosen under older rules (feature-carrying patches before 1.8.0, then minor bumps sized to keep the per-minor-line changelog tabs balanced). They are not renumbered. The plain rule applies from 1.14.0.

Mechanics stay unchanged: the version lives in the workspace `Cargo.toml` (`[workspace.package]`), bumps at cut time with the changelog move (`UPCOMING` → `RELEASES`, which also gives the release its date), and iOS build numbers / Android versionCodes increment independently per upload (see the submission `history.md` files).
