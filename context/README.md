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

**1.1.3** submitted 2026-06-28: iOS **build 51** in Apple review, Android
**versionCode 11** uploaded to the Play closed-testing track; server + web live on
prod at 1.1.3.

**Media day (2026-06-28):** a Reddit launch post drove **38 → 772 users in ~24h**
(665 joined that day, 738 active). The release shipped feedback-driven work — card
names while swiping, the deck-form overhaul (tap-to-pick fields, "Not set" empty
states, inline deck-name validation), expanded tags + format/power pickers, an
in-app privacy policy, and under-field form validation.

**Next:** keep the closed-testing 14-day clock running, land the in-flight QOL work
(draw-odds consistency stats, live swipe drag cues), and work the feature backlog.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
