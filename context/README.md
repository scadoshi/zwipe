# Context — Start Here

Orientation for AI assistants and returning contributors. This `context/` tree is
the project's living documentation; each subdirectory owns one concern.

## Directory map

| Directory | What's in it |
|-----------|--------------|
| [`product/`](product/) | What we're building — PRD, monetization, `premium/` feature catalog |
| [`architecture/`](architecture/) | Why the app is built this way — structure, decisions, hosting |
| [`development/`](development/) | How to write code here — commit/doc standards, newtypes, Dioxus, UI-text conventions |
| [`operations/`](operations/) | How to build, deploy & ship — `infrastructure/`, `ios/`, `android/` |
| [`marketing/`](marketing/) | Marketing material + tooling (business card, etc.) |
| [`plans/`](plans/) | Implementation plans for upcoming / in-flight work |
| [`progress/`](progress/) | Where we are — `overview.md`, `todo.md`, `backlog.md` |
| [`archive/`](archive/) | No longer active; kept for history |

Plus [`CLAUDE.md`](CLAUDE.md) — the authoritative rules/instructions for AI assistants.

## Current focus

**Last shipped — 1.2.0** (both stores, 2026-06-30, first minor bump since 1.1.0):
iOS build 54 + Android versionCode 15; server batch deployed to prod first, web
live. The 2026-06-30 analytics/tagging batch: hypergeometric **draw-odds**, the
**Synergy on/off** toggle, **power level + other-tags**, deck tags **85→117**,
the PDH commander fix, an `edhrec_rank` popularity index, proliferate→Counters,
the include/exclude filter guard, and the create/edit top-scroll fix. All wire
changes additive (`#[serde(default)]` / `Opdate` / `x-synergy-applied` header) so
old clients keep working. See the [`progress/overview.md`](progress/overview.md)
top entry for the full list.

**After this version:** track 1.2.0 store review, keep the closed-testing 14-day
clock running, suggestion-signal **Phase 3 (ranking)**, and draw-odds **Phase 4
(premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
