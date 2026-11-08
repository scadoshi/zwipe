# 286. Portfolio's orangui source line ranges are stale

State: merged | Owner: laptop | PR: portfolio#21 | Build seen: scottyfermo.com, 2026-11-07

## Report
Surfer: on scottyfermo.com/projects/orangui/, the `components.css#L…` anchors land mid-rule or on other components (Button's L171-L247 starts at `.stat-chip-muted`, Chip's L54 starts inside a rule, L1602 is mid-rule). The files exist; the line numbers drifted as orangui changed. Options: drop line ranges, or compute them at build time.

## Decision
Scotty, 2026-11-08: drop the line numbers; links point to the file only.

## Fix
laptop, portfolio#21 (merged 2026-11-08): all 10 orangui-page Source links point to the file only; no other #L anchors into orangui or zwipe. Test `source_links_point_to_whole_files`. Verify: Surfer opens each Source link on scottyfermo.com/projects/orangui/.
