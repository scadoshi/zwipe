# Context — Start Here

Orientation for AI assistants and returning contributors. This `context/` tree is
the project's living documentation; each subdirectory owns one concern.

## Directory map

| Directory | What's in it |
|-----------|--------------|
| [`product/`](product/) | What we're building — PRD, monetization, `premium/` feature catalog |
| [`architecture/`](architecture/) | Why the app is built this way — structure, decisions, hosting, content delivery (compiled vs fetched) |
| [`development/`](development/) | How to write code here — commit/doc standards, newtypes, Dioxus, UI-text conventions |
| [`operations/`](operations/) | How to build, deploy & ship — `infrastructure/`, `ios/`, `android/` |
| [`marketing/`](marketing/) | Marketing material + tooling (business card, etc.) |
| [`plans/`](plans/) | Implementation plans for upcoming / in-flight work |
| [`progress/`](progress/) | Where we are — `overview.md`, `todo.md`, `backlog.md` |
| [`archive/`](archive/) | No longer active; kept for history |

Plus [`CLAUDE.md`](CLAUDE.md) — the authoritative rules/instructions for AI assistants.

## Current focus

**Latest — 2026-08-12: 1.8.0 SUBMITTED to both stores; PATCH-only server
live.** The three-day arc: **1.7.5 released both stores 2026-08-10**, **1.7.6**
(global undo, featured flavor, Android resume-crash fix, quick-add-past-skips,
keyring 4) cut from the work Mac and released 2026-08-10/11, then **1.8.0**
(deck list Group by + Show chip rows, one-time deck-list tip, pinned
import/export consoles) cut and submitted 2026-08-12 — the first train under
the **any-feature-bumps-minor** convention
([`development/versioning.md`](development/versioning.md)). Same day:
`MIN_CLIENT_VERSION=1.7.5` raised and the **Phase 5 cleanup merged** (PR #24)
— PUT routes gone, legacy Opdate dialect deleted, PATCH is the only update
wire. Both plans archived. The work Mac is now a full build machine for both
platforms plus the dev-to-phone loop, and the repo lives at
`~/Developer/zwipe`.

**Watch items:** `zcripts/metrics/errors.sql` for stray-PUT canary rows (through
~08-14), the crash reporter for the ndk-context panic going silent on vc33+
sessions, 1.8.0 review/rollout, and the client-error-reporting prod
verification.

**After this:** next build candidates (owner to choose): the wasm build
blockers toward the full webapp (the strongest 1.9/2.0 anchor), social
features / featured decks, commander shortlist, fill basics — the activity
report's 15.3% deck-completion cliff (median 26 cards) points hardest at
fill-basics and composition targets. Then **Phase 6** — serve on the matured
otag signal (data-gated, months out). Ongoing: otag description authoring into
the tail, short-form marketing videos, review tracking, funnel numbers (gate
the sign-in-with-Google decision), and draw-odds **Phase 4 (premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
