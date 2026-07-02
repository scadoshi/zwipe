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

**On main, unreleased (2026-07-02):** the **CardFilter split** — `CardCriteria`
(shared predicate) + `CardQuery` (bounded server search, wire JSON unchanged) +
`Cards` (in-memory collection, no limit by construction) — plus the deck cards
screen defaulting to **alphabetical order**. Unpushed (push auto-deploys the
server); the client half rides the next release. See the
[`progress/overview.md`](progress/overview.md) top entry.

**Last shipped — 1.2.1** (client-only, 2026-07-01): a **card rules dialog** (util-bar
eye button → oracle text + stats with real mana/symbol glyphs, type/rarity/keyword
chips, P/T or loyalty; replaces the Keywords button, completing FR #8) and a **native
WebView launch-flash fix**. **Android versionCode 16** published to closed testing;
**iOS build 55** uploaded to App Store Connect, staged behind 1.2.0 (submit once
1.2.0 is live).

**After this version:** track 1.2.0 store review (then submit 1.2.1 on iOS), keep the
closed-testing 14-day clock running, suggestion-signal **Phase 3 (ranking)**, and
draw-odds **Phase 4 (premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
