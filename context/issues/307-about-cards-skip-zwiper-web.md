# 307. zwipe.net About's crate cards leave out zwiper-web

State: reported | Owner: - | PR: - | Build seen: 02102ab5e

## Report
Surfer, live zwipe.net/about, 2026-11-08, while verifying 294. The architecture diagram now has zwiper-web, and its caption says eight crates, but the crate cards under "How it fits together" are only seven: zwiper, zite, zerver, zwipe-ui, components, zwipe-client, zwipe-core. There's no card for zwiper-web, the web app.

## Repro
1. Open https://zwipe.net/about/.
2. Scroll past the diagram: the cards skip zwiper-web.
