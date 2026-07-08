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

**Latest — shared-components arc + portfolio adoption (2026-07-08):**
`zwipe-components` grew from 3 to 10 components in a day (`CardRow`,
`ThemePicker`, `NavDropdown`, `NavBar`, `PageMeta`, `OracleText`,
`KeywordChips` joined Button/Chip/ActionBar), now owns `themes.css`, and
exports `COMPONENTS_CSS`/`THEMES_CSS` for external consumers — because the
owner's **portfolio site became the crate's first external consumer** via a
GitHub git dependency (~760 duplicated lines deleted there; crates.io
declined). Same day: `zwipe_core::domain::site` centralized all base
URL/contact constants across app/site/server, zite shipped a polish batch
(ghost skeletons, featured-row role tags, dead-end not-shared screen,
guides sitemap + Article JSON-LD), and zwiper banked polish for the next
build (skeleton realignment, land-target filter-leak fix, DFC mana-cost
fallback). Details:
[`plans/components_portfolio_adoption.md`](plans/components_portfolio_adoption.md)
and the [`progress/overview.md`](progress/overview.md) top entries.

**1.4.0 LIVE on the App Store (2026-07-08; iOS build 61 / Android vc22):** the feature batch
— Zwipe-select popularity ordering, commander-select signal ingest, partner
autofill, Deck MVPs phase 1, deck share links. Server halves (three additive
migrations) deployed to prod first and verified against live clients. 1.3.0
(build 59 / vc20) and 1.3.1 (build 60 / vc21) superseded; the next build
carries the 2026-07-08 zwiper polish as **1.4.1** (or **1.5.0** if new
functionality lands first). Serve-path follow-ons
remain data-gated: [`plans/suggestion_signal.md`](plans/suggestion_signal.md)
(Phase 3c) and [`archive/commander_select_signal.md`](archive/commander_select_signal.md)
(§4 Consumer B).

**After this:** short-form videos (`marketing/plans/` — the share/MVP/
commander videos are filmable now against 1.4.0), review tracking (then bump
`MIN_CLIENT_VERSION`), keep the closed-testing 14-day clock running, watch
the funnel numbers (they gate the sign-in-with-Google decision), privacy
follow-ups (store data-safety labels + notification email), and draw-odds
**Phase 4 (premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
