# 296. A collapsed group's tag loses its bottom border color on hover

State: merged | Owner: laptop | PR: #433 | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot): a collapsible group's tag ("▸ All (13)", among chips like Lands Matter and Proliferate), once collapsed and hovered, shows the purple hover outline on top and sides but the bottom edge stays grey. Likely the expanded state's bottom-border override (joining the tag to its open list) still applies when collapsed. Find every collapsible group tag that shares the style, app and web.

## Fix
Branch `nits/laptop-296`. The tag is a Decks group chip (`.deck-group > .card-group-header.group-collapsible`, Group by Tags, Color identity or Format, app and web share it). Folded, the generic `.card-group.collapsed .card-group-header` rule cleared every group header's bottom edge, and a second rule (`.card-group.deck-group.collapsed > .card-group-header.group-collapsible`, five classes) put the chip's bottom edge back to the resting border color. That rule outranked web's hover rule (`.card-group-header.group-collapsible:hover`), so on hover the top and sides lit and the bottom stayed. Now the generic rule skips deck groups (`.card-group.collapsed:not(.deck-group)`) and the override is gone, so nothing stands between the chip and the hover. Open chips were already right. No other collapsible group header is a chip: the deck cards screen's headers have only a bottom rule, which still fades when folded.
