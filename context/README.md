# Context: Start Here

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

Plus [`CLAUDE.md`](CLAUDE.md), the authoritative rules for AI assistants.

## Where things stand

This section is a pointer, not a second copy. The running log lives in
[`progress/overview.md`](progress/overview.md), newest entry first, and that is
the file to update when something ships.

As of 2026-09-21: 1.10.1 is live on both stores (iOS build 79 / Android
versionCode 42, live 2026-09-07). Prod survived a 53-hour outage 09-11 → 09-13;
the hardening that closes it is done, monitoring included. The `zerver` feature
flag is gone as of 09-21, so no client depends on the server crate any more.

What needs a human, all tracked in [`progress/todo.md`](progress/todo.md):
re-read the error and crash tables by 2026-10-06, rotate the prod db password,
reinstall Zwipe from Play on the Pixel, and eyeball a store build against
1.10.1.

See [`progress/feature_requests.md`](progress/feature_requests.md) for the
weighted request queue.
