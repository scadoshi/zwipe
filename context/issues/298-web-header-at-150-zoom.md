# 298. The web app's header breaks at 150% zoom

State: reported | Owner: - | PR: - | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot, app.zwipe.net/decks at 150% browser zoom, window ~1200 CSS px wide at that zoom): the header wraps into two rows, nav pills (Decks, Playstyle, Activity, Commander maybeboard) on top and the icon row plus Log out underneath, with an empty dark block filling the left of the header beside the logo and Back. The header takes about a third of the viewport. "A little crazy."

## Repro
Surfer to reproduce: Chrome zoom 125%, 150%, 175% (or the equivalent CSS widths) on Decks and a couple of other screens. Note the widths where the header wraps, what the empty block is, and whether the deck page and sheets have the same problem. Shots to halo ~/Developer/surfer-shots/298-*.
