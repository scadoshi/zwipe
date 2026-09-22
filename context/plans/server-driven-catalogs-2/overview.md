# Server-driven catalogs, part 2: zwiper reads nothing compiled in

**Status: DONE 2026-09-22, all three phases, riding 1.10.2.**

zwiper reads no compiled catalog. Verified by grep: `selectable_franchises`, `FRANCHISES`, `keyword_reminder(`, `HttpChangelog::current` and `CURATED_ORACLE_TAGS` all return zero hits in `zwiper/src`. `ALLOWED_THEMES` returns one, which is the exception and the point.

Continues [`../archive/server_driven_catalogs.md`](../archive/server_driven_catalogs.md), which made card roles and deck tags server-driven in July. That plan solved two catalogs. This one states the rule behind it and finishes the job.

## The rule (owner, 2026-09-22)

> Compile-time tables can remain, but only zite and zerver read them. zwiper gets everything served from zerver.

The reasoning is that zwiper cannot do anything useful offline. Every screen needs the API, so a compiled fallback renders data the user cannot act on: the Universes Beyond picker writes a preference, keyword chips decorate cards that came from a search, and the changelog is a screen you reached by opening an app that already failed to reach the server. Caching for an operation that can never complete is code with no reachable path.

The second reason is latency. Anything zwiper compiles in can only change on a store train. Anything it fetches changes on a deploy, which is minutes.

## Inventory

Audited 2026-09-22 by grepping every compiled table in `zwipe-core` against each crate that reads it.

| Table | zwiper today | Phase |
|---|---|---|
| `selectable_franchises` (Universes Beyond) | served, phase 1 | done |
| `keyword_reminder` (343 entries) | served, phase 2 | done |
| `HttpChangelog::current` | served, phase 2 | done |
| `CURATED_ORACLE_TAGS` (48 slugs) | served as a field, phase 3 | done |
| `ALLOWED_THEMES` | compiled, and **stays** | n/a |

`ALLOWED_THEMES` is the principled exception. The themes are CSS compiled into the binary, so a served list would name palettes the app does not have. The list and the thing it describes ship together, which is exactly the case where compiling in is correct.

Nothing else qualifies. Types, validation and business rules stay in `zwipe-core` and stay compiled: they change with the binary by definition, so serving them would buy nothing.

## What does not move

The tables themselves stay in `zwipe-core`. zerver serves from them and zite renders them (`HttpChangelog::current` backs zite's static changelog page, and the keyword reminders back its card details). Only zwiper's reads go away, so this deletes branches, not data.

That also means the app binary does not shrink. The win is latency and one less state to reason about, not size.

## Phases

1. **[Universes Beyond franchises](ub_franchises.md)** — new endpoint. Most of it is built already.
2. **[Drop the two fallbacks](fallback_removals.md)** — keyword reminders and the changelog. Deletions only, no new endpoints.
3. **[Curated oracle tags](curated_oracle_tags.md)** — new endpoint, same shape as phase 1.

Order matters only in that phase 1 establishes the shape the others copy. Phases 2 and 3 are independent of each other.

## Verification

- Every phase: full CI gate per `../../development/commit_guidelines.md`.
- Per phase: with the backend stopped, the affected screen shows its failure state and no stale compiled content.
- After all three: `grep` zwiper for the four table names returns nothing. `ALLOWED_THEMES` should still be there, and that is the point.
- Device pass before the cut: open the UB picker, a card with keyword chips, the changelog screen, and an otag picker. All four are read paths, so a broken fetch is visible rather than silent.

## Wire safety

Every phase is additive on the server: new endpoints, no changed responses. Shipped clients keep reading their compiled copies and never call the new routes. Nothing here needs a `MIN_CLIENT_VERSION` bump.
