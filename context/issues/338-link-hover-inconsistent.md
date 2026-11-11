# 338. Text links hover differently across the three sites

State: fixing | Owner: thinker | PR: - | Build seen: live 7d3edf896

## Report
Scotland, 2026-11-11, measured live: links that are accent at rest (the footer note, app.zwipe.net's tagline) only thicken their underline on hover. zwipe.net's body links (About, guides) hover to a lighter mixed color, not the accent. The portfolio's project links (Cairn, Zwipe, App Store) get no underline.

## Diagnosis
thinker, 2026-11-11. orangui (pinned at 67505e9) already defines the text-link hover: an unclassed `a` takes the accent and a 2px underline on hover and on keyboard focus, at specificity (0,2,0). app.zwipe.net and the shared footer note follow it.
- **zite:** its own `a:not(.tag):hover` at (0,2,1) outranks the kit and mixes 30% of `--palette-1` into the text color. That's the lighter hover on the tagline, the banner's "See what's new" and the About and card links.
- **Portfolio** (scottyfermo.com): resting `a { text-decoration: none }` strips the underline, and the same brighten rule wins the hover.
- Measured with headless Chrome forced to a hover-capable pointer and `:hover` forced on each link.

## Fix
One rule everywhere: a text link is its ink, underlined with a 3px offset, at rest, then the accent with a 2px underline on hover and focus. orangui needs no change.
- **zite** (this PR): the brighten-on-hover rule only applies to classed links (cards, the brand, pills), so text links fall through to the kit's hover.
- **Portfolio** (its own PR): unclassed links get the resting underline, the brighten rule skips them, and the footer's classed "This site" link takes the same hover.
- On gruvbox the ink and the accent are one color, so the hover's visible change there is the thicker underline; on themes where they differ, the color moves to the accent too.
