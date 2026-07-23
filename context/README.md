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

**Latest — server 1.7.3 deployed (2026-07-23); client build not yet cut.** This batch
(server + zite live; client bits ride the next build): **sqlx 0.9** in prod (feature
split + regenerated `.sqlx/`, gated by the integration suite), a **deps refresh**
(115 semver bumps), the **filter sheet current/staged split** (Apply is the only
commit; Reset/Cancel stage + restore, with toasts), **average P/T** in the deck's
Distributions, the **shared ranked otag search** in core (exact > slug/label >
description) powering the selector, card filter, and dictionary, and the
**share-page group-ordering fix** (live). The changelog's 1.7.3 **Upcoming teaser**
is live in-app; store what's-new logs are started for both stores.
Details: [`progress/overview.md`](progress/overview.md) top entry.

**1.7.2 LIVE on both stores** (submitted 2026-07-20): board-wide filters, pinned
lands section, dialog backdrop-dismiss, guides polish, shared-deck tokens.

**After this:** cut **1.7.3 clients** when the batch feels full (Android must bump
`targetSdk` by **2026-08-31** — fold it into this or the next release). Once `<1.7.0`
clients drain, floor `MIN_CLIENT_VERSION` to 1.7.0 → unlocks the Phase 5S step-3
cleanup (drop the legacy commander wire + fallback). Then **Phase 6** — serve on the
matured otag signal (data-gated, months out). Ongoing: description authoring into the
tail (runbook at [`development/runbooks/`](development/runbooks/)), short-form
marketing videos, review tracking, funnel numbers (gate the sign-in-with-Google
decision), privacy follow-ups (store data-safety labels + notification email), and
draw-odds **Phase 4 (premium gating)**. Queued features: share-page charts, mana
pip-count filter, commander shortlist, deck folders.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
