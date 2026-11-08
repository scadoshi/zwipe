# 296. A collapsed group's tag loses its bottom border color on hover

State: approved | Owner: laptop | PR: - | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot): a collapsible group's tag ("▸ All (13)", among chips like Lands Matter and Proliferate), once collapsed and hovered, shows the purple hover outline on top and sides but the bottom edge stays grey. Likely the expanded state's bottom-border override (joining the tag to its open list) still applies when collapsed. Find every collapsible group tag that shares the style, app and web.
