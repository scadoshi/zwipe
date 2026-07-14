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

**Latest — server 1.7.0 pushed (2026-07-14); client build pending.** Shipped this
batch: the in-app **oracle-tag dictionary** (letter-first browse + search over all
~4,500 tags with our descriptions), a **unified catalog cache** (filters/pickers
prefetched at startup → instant, no reload flicker), **1,100 authored descriptions**
(oracle-text-verified; high-traffic head fully covered), the **Phase M sunset**
(`mechanical_categories → card_roles` everywhere incl. a DB-column rename migration),
and **Phase 5S dual-accept** (the swipe signal is now `deck_id`-driven server-side —
non-EDH collects the same as EDH — with a legacy fallback so 1.6.0 clients still land
signal). Verified push-safe for live 1.6.0 clients (no `MIN_CLIENT_VERSION` bump).
Details: [`progress/overview.md`](progress/overview.md) top entry, [`plans/otags/`](plans/otags/).

**1.6.0 LIVE (2026-07-12; iOS build 64 / Android vc26):** the big feature batch —
in-app changelog, the three-axis tag system (oracle tags / card roles / deck tags,
server-driven via slugs), 17 new themes (31 total), persisted theme across app +
site, home-screen buy links, and a reorganized deck view.

**After this:** the **1.7.0 clients are submitted for review** (iOS build 65, Android
vc27; live in ~1 day) carrying the dictionary, faster filters, and the `deck_id`-only
signal. Once live + adopted, floor `MIN_CLIENT_VERSION` to 1.7.0 → unlocks the Phase 5S
step-3 cleanup (drop the legacy commander wire + fallback). Then **Phase 6** — serve on
the matured otag signal (data-gated, months out). Ongoing: description authoring into the tail
(runbook at [`development/runbooks/`](development/runbooks/)), short-form marketing
videos, review tracking, funnel numbers (gate the sign-in-with-Google decision),
privacy follow-ups (store data-safety labels + notification email), and draw-odds
**Phase 4 (premium gating)**. Small queued polish: otag-selector search over
descriptions + a Dictionary link on the card filter; optional dialog backdrop-dismiss.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
