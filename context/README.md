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

**Latest — 2026-08-05: 1.7.5 SUBMITTED to both stores, in review** (iOS build
70 / Android vc32, cut from the home Mac; the work Mac became a second build
machine the same day). The release, mostly on the deck cards screen: **quick
add** (type-to-search, tap to add), the **deck identity header**, **floating
type-to-search results** across all pickers, **quantity debounce** (one net
call per tap burst), and **undo** (adds, removals, qty bursts, board moves,
printing swaps — per-deck stacks that survive navigation). It also carries the
client half of the **PATCH migration** (PATCH verb everywhere, absolute
quantities, clean Opdate wire); all three server layers are already deployed
and verified on prod. **After rollout:** raise `MIN_CLIENT_VERSION=1.7.5`,
quiet days, then the Phase 5 cleanup — plan
[`plans/patch_idempotent_updates.md`](plans/patch_idempotent_updates.md).
Also 2026-08-05: the nightly-zervice **double-run bug excised** (cron ghost
survived the July timer migration; deadlock postmortem in
`operations/infrastructure/server.md`).

**1.7.4 LIVE on both stores** (submitted 2026-07-30): **client error + crash
reporting** (anonymous, first-party — its first real catch, the Android
ndk-context resume crash, is triaged in todo) and the iOS photo-save crash
fix. Privacy policy + both stores' data-safety declarations updated.

**1.7.3 LIVE** (2026-07-24): filter sheet current/staged split, average P/T,
shared ranked otag search, otag definitions in the swipe details dialog;
first targetSdk 36 Android build.

**After this:** owner is building **global undo**
([`plans/global_undo.md`](plans/global_undo.md)) — one per-deck mutation
history across screens. Then the review-window quick wins (429 copy rounding,
solid-background favicon, search-bar clear buttons, contribute page) and the
**Android resume-crash fix** for 1.7.6. Then **Phase 6** — serve on the
matured otag signal (data-gated, months out). Ongoing: description authoring
into the tail (runbook at [`development/runbooks/`](development/runbooks/)),
short-form marketing videos, review tracking, funnel numbers (gate the
sign-in-with-Google decision), and draw-odds **Phase 4 (premium gating)**.
Queued features: flavor rotation, share-page charts, mana pip-count filter,
commander shortlist, deck folders.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
