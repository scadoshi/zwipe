# zwiper-web

Zwipe in a browser. Every screen of the phone app (zwiper) is here, from Login to the swipe screens, laid out for a big window: a sidebar on the left, panels side by side, toolbars under the header, keys for the swipes. Below 900px wide it falls back to the app's single column.

It is a client-rendered Dioxus app and its own crate. It talks to zerver through `zwipe-client`, takes its screen logic from `zwipe-ui` and draws with `components` (and orangui under it), so copy, rules and look come from the same places the app uses.

More reading in `context/`: `parity-plan.md` (what was built, the kept deviations, the launch blockers), `web-layout.md` (how each screen sits on a wide window), `ARCHITECTURE.md` (how the crate fits together), `audit.md` (the last screen-by-screen check against the specs) and `specs/` (the app's behavior, the source of truth).

## Run it

You need `dx` (the Dioxus CLI, 0.7.10), the `wasm32-unknown-unknown` target, and a zerver to talk to.

1. Start a local zerver (`cargo run --bin zerver` from the repo root, set up per the root `CLAUDE.md`). Its `ALLOWED_ORIGINS` must include `http://localhost:8080`, or the browser blocks every request. An env override works as well as an edit to `zerver/.env`:

```bash
ALLOWED_ORIGINS=http://localhost:8080 cargo run --bin zerver
```

2. From this folder:

```bash
BACKEND_URL=http://localhost:3000 dx serve
```

3. Open http://localhost:8080 and log in with an account on that zerver.

`BACKEND_URL` is read at build time. Without it a debug build talks to `http://localhost:3000` and a release build to `https://api.zwipe.net`. The session, the last theme, the last counters and Home's count scope live in the browser's `localStorage` for this origin.

`dx build --platform web` makes the bundle without serving it. The gate every commit passes is in `context/parity-plan.md`.

## Before it ships

Two things block a public launch, and both need zerver changes:

- The refresh token sits in `localStorage`, where any script on the origin can read it. It should move into an httpOnly cookie that zerver sets and reads.
- The anonymous funnel events (app opened, Register viewed, Register submitted) are wired but held off by `SEND_FUNNEL_EVENTS` in `src/lib/inbound/session/funnel.rs`. Zerver first has to record which platform sent each one, or web visits would land in the app's funnel numbers.

Merging `feat/web` into main is the owner's call.
