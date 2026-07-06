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

**Latest — 1.3.1, built 2026-07-05 (ship pending):** first **pre-registration
funnel instrumentation** — anonymous, PII-free session events (`app_opened`,
`register_viewed`, `register_submitted`) posted to a new unauthenticated
endpoint, so install→register drop-off becomes a query instead of a guess.
Rides with the server-side **service type-erasure** refactor (`AppState` and
all handlers drop their generic params; no behavior change) and a
`user_daily_activity` BIGINT widening. **iOS build 60** / **Android vc21**
signed and ready; server (two additive migrations) deploys first, then store
upload. See the [`progress/overview.md`](progress/overview.md) top entry.

**Before that — 1.3.0, submitted 2026-07-02:** per-swipe durable skips,
per-deck stack memory, the CardStack refactor, profile About section, and the
filter-intent + Reset pass (iOS build 59 / Android vc20, in store review).

**After this version:** short-form videos against the 1.3.x build
(`marketing/plans/` — #2 and #4 are both filmable now), track review (then
bump `MIN_CLIENT_VERSION`), keep the closed-testing 14-day clock running,
watch the new funnel numbers (they gate the sign-in-with-Google decision),
privacy follow-ups (store data-safety labels + notification email),
suggestion-signal **Phase 3 (ranking)**, and draw-odds **Phase 4 (premium
gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
