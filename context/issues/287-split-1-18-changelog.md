# 287. Split the 1.18 changelog into logical patches

State: approved | Owner: laptop | PR: - | Build seen: -

## Report
Scotty, 2026-11-07: 1.18.0 (79 entries, shipped Nov 5) and the upcoming 1.18.1 (35 entries) are too big for the changelog sheet. Regroup them into reasonably sized, logically grouped patches: 1.18.0 becomes 1.18.0, 1.18.1, … 1.18.k (all dated Nov 5, 2026, since they shipped together), and the upcoming work becomes 1.18.(k+1) … 1.18.N in `UPCOMING`. The next store build ships as 1.18.N. Store history doesn't change; this is purely how the changelog reads.

## Notes
- Source: `zwipe-core/src/content/changelog/mod.rs`. Every entry keeps its exact text; only its version moves. Rules: `context/development/changelog_guidelines.md`.
- Tests to keep green: newest first by version and date, distinct versions, upcoming newer than the latest release, `workspace_version_is_latest_or_upcoming` (the workspace stays 1.18.0 until cut; set it to 1.18.N at cut time per `versioning.md`).
- zwipe.net's home banner shows `RELEASES[0]`, so it will name 1.18.k while the stores show 1.18.0. Scotty is fine with the history rewrite.

## Fix
Branch `nits/laptop-287`. 1.18.0's 78 entries become 1.18.0 to 1.18.8, all dated Nov 5, 2026, and 1.18.1's 35 upcoming entries become 1.18.9 to 1.18.12 in `UPCOMING`. Groups by theme, 6 to 10 entries each, with the table in the PR. Every entry's text is unchanged (a sorted diff of all entries before and after is empty), and each group keeps its entries' original order. The workspace version moves to 1.18.8 so `workspace_version_is_latest_or_upcoming` holds (it must be the latest release or an upcoming one); at cut time it becomes 1.18.12.
