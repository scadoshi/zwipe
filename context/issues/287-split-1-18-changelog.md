# 287. Split the 1.18 changelog into logical patches

State: approved | Owner: laptop | PR: - | Build seen: -

## Report
Scotty, 2026-11-07: 1.18.0 (79 entries, shipped Nov 5) and the upcoming 1.18.1 (35 entries) are too big for the changelog sheet. Regroup them into reasonably sized, logically grouped patches: 1.18.0 becomes 1.18.0, 1.18.1, … 1.18.k (all dated Nov 5, 2026, since they shipped together), and the upcoming work becomes 1.18.(k+1) … 1.18.N in `UPCOMING`. The next store build ships as 1.18.N. Store history doesn't change; this is purely how the changelog reads.

## Notes
- Source: `zwipe-core/src/content/changelog/mod.rs`. Every entry keeps its exact text; only its version moves. Rules: `context/development/changelog_guidelines.md`.
- Tests to keep green: newest first by version and date, distinct versions, upcoming newer than the latest release, `workspace_version_is_latest_or_upcoming` (the workspace stays 1.18.0 until cut; set it to 1.18.N at cut time per `versioning.md`).
- zwipe.net's home banner shows `RELEASES[0]`, so it will name 1.18.k while the stores show 1.18.0. Scotty is fine with the history rewrite.
