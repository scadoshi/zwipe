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

**Latest — 1.3.2, server-only, LIVE 2026-07-06:** **adaptive serve
ordering** — the default synergy stack scores cards by the scraped base plus
Zwipe's own swipe signal (net add-rate: adds full credit, maybes half,
removals negative, skips as denominator drag), then deals them in **bands of
25** (one page = one hand) shuffled per (deck, day). A different opening hand per deck per day, the
same deck stable within a day, and crowd favorites break into the opening
hand as signal accrues. Direct response to user feedback about repetitive
serving. See the [`progress/overview.md`](progress/overview.md) top entry and
[`plans/suggestion_signal.md`](plans/suggestion_signal.md) (Phase 3c remains,
data-gated).

**In review — 1.3.1 (iOS build 60 / Android vc21, submitted 2026-07-06):**
pre-registration funnel instrumentation (anonymous, PII-free session events →
install→register drop-off becomes a query; the numbers gate the
sign-in-with-Google decision), plus the server-side type-erasure refactor and
daily-activity BIGINT widening. 1.3.0 (build 59 / vc20) also still in review.

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
