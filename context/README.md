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

**Latest — 1.2.3, submitted 2026-07-02:** **swipe memory** (FR #11 — per-deck
skip/removal suppressions the server stops serving, "Clear skips" in the deck
More sheet, plus per-user + weekly signal tables now collecting in prod;
`plans/swipe_memory.md`), the **CardFilter split** (`CardCriteria` + `CardQuery`
+ `Cards`, wire JSON unchanged), **alphabetical deck lists**, a profile
System/version row, the email-verification row rework, and an updated privacy
policy (per-account activity disclosed). Server + zite deployed 2026-07-02;
**iOS build 56** and **Android versionCode 17** in store review. 1.2.2 was
skipped. See the [`progress/overview.md`](progress/overview.md) top entry.

**After this version:** track 1.2.3 review (then bump `MIN_CLIENT_VERSION`),
keep the closed-testing 14-day clock running, privacy follow-ups (store
data-safety labels + notification email), suggestion-signal **Phase 3
(ranking)** — now with per-user data accruing — and draw-odds **Phase 4
(premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
