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

**Latest — oracle-tag mapping sweep + description pipeline (2026-07-13):** a
machine-assisted audit of the two hand-authored otag mapping tables shipped —
**Track A** retuned 40 archetype seed sets (dropping over-broad seeds, adding
high-signal ones), **Track B** expanded the card-role roots/overrides, and a new
**`ROLE_TAG_EXCLUSIONS`** mechanism subtracts mis-parented tags per-role (what
subtree-root narrowing can't do, since a tag can sit under several roots at once);
all validated against a local `zervice` recompute. Same day, the **oracle-tag
description pipeline** (F Part 1) shipped: `zervice` overlays our authored
descriptions into `oracle_tags.description` each sync (ours always wins, survives
the daily nuke). Plus small polish — the swipe eyeball dialog's Flip moved to its
footer with a single scroll, an Export-screen skeleton, and a zite banner/hamburger
overlap fix hoisted into shared `zwipe-components`. Details in the
[`progress/overview.md`](progress/overview.md) top entry and
[`plans/otags/`](plans/otags/).

**1.6.0 LIVE (2026-07-12; iOS build 64 / Android vc26):** the big feature batch —
in-app changelog, the three-axis tag system (oracle tags / card roles / deck tags,
server-driven via slugs), 17 new themes (31 total), persisted theme across app +
site, home-screen buy links, and a reorganized deck view. Android production launch
was submitted for review 2026-07-11 (all countries), awaiting Google.

**After this:** **1,100 oracle-tag descriptions** are now authored (oracle-text-verified;
high-traffic head fully covered) — next is the in-app **dictionary page** that renders
them ([`plans/otags/tag_descriptions_and_dictionary.md`](plans/otags/tag_descriptions_and_dictionary.md),
authoring runbook at [`development/runbooks/`](development/runbooks/)), tail authoring
continues in the background, then **serve the changelog from the server** so pipeline/
release notes no longer need an app resubmit ([`plans/changelog_server.md`](plans/changelog_server.md)).
Ongoing: short-form marketing videos, review tracking (then bump
`MIN_CLIENT_VERSION`), watch the funnel numbers (they gate the sign-in-with-Google
decision), privacy follow-ups (store data-safety labels + notification email), and
draw-odds **Phase 4 (premium gating)**.

See [`progress/overview.md`](progress/overview.md) for the high-level state,
[`progress/feature_requests.md`](progress/feature_requests.md) for the weighted
request queue, and [`progress/todo.md`](progress/todo.md) for the ordered task list.
