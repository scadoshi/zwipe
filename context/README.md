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

**Next version — staged on `main`, building now (2026-06-29).** A 24-commit batch
of deck-building tooling (land target, price filter, price budget), the
first-party **suggestion signal** (`commander_card_signal` — now collecting), the
collapsible deck-view sections, and UI polish (filter button alignment,
bottom-sheet flash fix, home flavor header). All backward-compatible; **server
slices deploy first** (three additive nullable migrations). See the
[`progress/overview.md`](progress/overview.md) top entry for the full list.

**Last shipped — 1.1.3** (both stores, 2026-06-28, media-day release): iOS build
51 + Android vc11; server + web live on prod. A Reddit launch post drove **38 →
772 users in ~24h**. Shipped card names while swiping, the deck-form overhaul,
expanded tags + format/power pickers, an in-app privacy policy, and under-field
validation.

**After this version:** keep the closed-testing 14-day clock running, suggestion-
signal **Phase 3 (ranking)**, and the in-flight QOL work (draw-odds, drag cues).

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
