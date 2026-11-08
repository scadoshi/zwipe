# 307. zwipe.net About's crate cards leave out zwiper-web

State: fixing | Owner: laptop | PR: #447 | Build seen: 02102ab5e

## Report
Surfer, live zwipe.net/about, 2026-11-08, while verifying 294. The architecture diagram now has zwiper-web, and its caption says eight crates, but the crate cards under "How it fits together" are only seven: zwiper, zite, zerver, zwipe-ui, components, zwipe-client, zwipe-core. There's no card for zwiper-web, the web app.

## Repro
1. Open https://zwipe.net/about/.
2. Scroll past the diagram: the cards skip zwiper-web.

## Fix
Branch `nits/laptop-307`. zwipe.net/about gets a zwiper-web card after zwiper's ("Web app": the app on the web, at app.zwipe.net, talks to zerver, imports zwipe-core, zwipe-client, zwipe-ui and components, as its Cargo.toml does), the section summary says eight crates instead of seven, and zwipe-client's card names the web app among its callers. New tests in `zite/src/pages/about.rs`: `the_crate_cards_are_the_workspace_members` (the section's card titles must equal the root Cargo.toml's members) and `the_summary_counts_the_crates`, alongside components' caption test. The portfolio's Zwipe page already lists all eight crates and its diagram already has zwiper-web, so nothing changes there.
