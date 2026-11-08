# 294. Portfolio's Zwipe page leaves out the web app

State: merged | Owner: thinker | PR: portfolio#22, #434 | Build seen: scottyfermo.com, 2026-11-08

## Report
Scotty, 2026-11-08 (screenshot of the Architecture slide, "How it fits together"): the diagram shows zwiper (iOS and Android) and zite (zwipe.net) calling zerver, and the caption says "Two clients, one server". The web app (zwiper-web, app.zwipe.net) is missing. Eventually update the Zwipe portfolio page to include it: the diagram, the caption, and any copy that counts clients. Not scheduled.

Also slide 1 (crate diagram, "Seven crates in one workspace..."): zwiper-web is missing there too, and the caption says "both apps". Scotty: the chart is getting complicated. Composer suggestion: one apps box (zwiper and zwiper-web: iOS, Android, web) instead of a new node, so neither slide grows.

## Decision
Scotty, 2026-11-08: add zwiper-web as its own new box on both slides for now; he will say if it gets too busy.

Scotty on the first draft: each client points at one crate, so it looks like it only pulls from that one. Redo: the clients join into one arrow at the shared group as a whole; zerver points at zwipe-core only.
