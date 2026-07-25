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

**Latest — 1.7.3 clients submitted (2026-07-24; iOS build 68 / Android vc30, live in
~1 day).** The batch: the **filter sheet current/staged split** (Apply is the only
commit; Reset/Cancel stage + restore, with toasts), **average P/T** in the deck's
Distributions, the **shared ranked otag search** in core (exact > slug/label >
description), and **otag definitions in the swipe-screen details dialog**. Android
vc30 is the first **targetSdk 36** build, clearing the Play deadline (2026-08-31).
Server side went ahead 2026-07-23: **sqlx 0.9** in prod, a **deps refresh** (115
semver bumps), and the share-page ordering fix. Changelog shows 1.7.3 as shipped;
the zite banner announces it; the iOS Description got a full refresh.
Details: [`progress/overview.md`](progress/overview.md) top entry.

**1.7.2 LIVE on both stores** (submitted 2026-07-20): board-wide filters, pinned
lands section, dialog backdrop-dismiss, guides polish, shared-deck tokens.

**After this:** `MIN_CLIENT_VERSION` floored to 1.7.0 (2026-07-24) → the **Phase 5S
step-3 cleanup** is next up (drop the legacy commander wire + fallback; the client
half rides the next build). Then **Phase 6** — serve on the
matured otag signal (data-gated, months out). Ongoing: description authoring into the
tail (runbook at [`development/runbooks/`](development/runbooks/)), short-form
marketing videos, review tracking, funnel numbers (gate the sign-in-with-Google
decision), privacy follow-ups (store data-safety labels + notification email), and
draw-odds **Phase 4 (premium gating)**. Queued features: share-page charts, mana
pip-count filter, commander shortlist, deck folders.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
