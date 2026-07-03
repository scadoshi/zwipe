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

**Latest — 1.3.0, submitted 2026-07-02:** supersedes 1.2.3 (withdrawn from
both stores pre-review; its swipe-memory feature set ships here). Adds
**per-swipe durable skips** (dedicated skip/unskip endpoints replace the lossy
usage-batch path), **per-deck stack memory** (every deck's add stack resumes
exactly where swiping left off, undo included — MRU parked per deck), the
**CardStack refactor** (one generic stack type + per-stack action models
across search/maybeboard/remove), and a visual polish pass (image/skeleton
ease-ins, enforced swipe-layout spacing) plus a profile **About section**
(website link + privacy + version). Server deployed 2026-07-02; **iOS build 58**
and **Android versionCode 19** in store review (57/18 re-submitted as 58/19 with
the About section). See the [`progress/overview.md`](progress/overview.md) top entry.

**After this version:** short-form videos against the 1.3.0 build
(`marketing/plans/` — #2 and #4 are both filmable now), track review (then
bump `MIN_CLIENT_VERSION`), keep the closed-testing 14-day clock running,
privacy follow-ups (store data-safety labels + notification email),
suggestion-signal **Phase 3 (ranking)**, and draw-odds **Phase 4 (premium
gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
