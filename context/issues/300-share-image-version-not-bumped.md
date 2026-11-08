# 300. Share image URL keeps its old ?v= after the image layout changes

State: fixing | Owner: thinker | PR: #438 | Build seen: 1d7053e88

## Report
Surfer, 2026-11-08. Surfer test's share page links `og:image` to `…/preview.png?v=1794023263` both before the 285 part 2 deploy and after it, while the PNG behind that URL changed to the new layout. Discord, X, iMessage and other link unfurlers cache images by URL, so a deck shared before the deploy keeps showing the old card there until the deck itself changes. The `v` looks like the deck's last update only; adding a render version to it would make a layout change refresh every preview.

## Repro
1. `curl -s https://zwipe.net/deck/f1c34d48-e388-477c-9b1a-74f1e56f6216 | grep og:image` before and after a preview layout deploy.
2. The `v` is the same both times; the image differs.
