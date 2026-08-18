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

**Latest — 2026-08-17: 1.9.2 SUBMITTED to both stores** (iOS build 76 /
Android versionCode 39), and it is the first build carrying a fix for the
Android `ndk-context` crash that had survived five releases. Root cause was
never the resume path: `MainActivity` had no `launchMode`, so an explicit
component start created a *second* Activity in a live process and re-ran
NativeActivity's native init. A second bug found the same day (`configChanges`
omitted `uiMode`, so a system theme change tore the Activity down and the
onDestroy process-kill silently closed the app) is fixed too. Both are applied
post-bundle by `zcripts/android/patch_bundle.sh`, and skipping that script
silently reships the crash. 1.9.2 also carries the back-swipe overlay fixes,
the deck list restyle with command-zone art, per-combination color grouping
with mana pips, and the zite work (guides search, Panel heroes, 36 guide
screenshots).

**Then 2026-08-18:** oracle-tag descriptions finished at **4,521 of 4,522**
(`nanni` deliberately blank), which also evicted the last of Scryfall's copy
and the markdown cross-links that came with it.

**Watch items:** 1.9.2 clearing review at both stores, then a **field check one
week after it goes LIVE** (not after submission) confirming the Android crash
fix held, run against both the crash table *and* Android session volume so a
drop in crashes can't be mistaken for a fix when it is really a drop in users.
Also: the Pixel needs a reinstall from Play once 1.9.2 is live, since it is
currently on a debug-signed build Play cannot update.

**After this:** next build candidates (owner to choose): the wasm build
blockers toward the full webapp (the strongest 2.0 anchor), social features /
featured decks, commander shortlist, import printings. The activity report's
15.3% deck-completion cliff (median 26 cards) points hardest at composition
targets; fill basics was the other answer to it and was **declined 2026-08-18**
([`plans/archive/fill_basics.md`](plans/archive/fill_basics.md)), so do not
re-propose it. Then **Phase 6** — serve on the matured otag signal (data-gated,
months out). Ongoing: short-form marketing videos, review tracking, funnel
numbers (gate the sign-in-with-Google decision), and draw-odds **Phase 4
(premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
