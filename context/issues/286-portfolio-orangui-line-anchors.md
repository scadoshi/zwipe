# 286. Portfolio's orangui source line ranges are stale

State: reported | Owner: - | PR: - | Build seen: scottyfermo.com, 2026-11-07

## Report
Surfer: on scottyfermo.com/projects/orangui/, the `components.css#L…` anchors land mid-rule or on other components (Button's L171-L247 starts at `.stat-chip-muted`, Chip's L54 starts inside a rule, L1602 is mid-rule). The files exist; the line numbers drifted as orangui changed. Options: drop line ranges, or compute them at build time.
