# zwiper-web

Every Zwipe app screen for a big browser window: a Dioxus web app with a sidebar and side-by-side panels, falling back to the app's single column below 900px. It talks to zerver through `zwipe-client` and draws with `components` (and orangui under it), so it looks and behaves like the app.

Status and plan: `context/parity-plan.md`. How each screen lays out: `context/web-layout.md`. How the crate fits together: `context/ARCHITECTURE.md`.

## Run it

You need `dx` (the Dioxus CLI, 0.7.10), the `wasm32-unknown-unknown` target, and a zerver to talk to.

1. Start a local zerver (`cargo run --bin zerver` from the repo root, set up per the root `CLAUDE.md`). Its `ALLOWED_ORIGINS` in `zerver/.env` must include `http://localhost:8080`, or the browser blocks every request.
2. From this folder:

```bash
BACKEND_URL=http://localhost:3000 dx serve
```

3. Open http://localhost:8080 and log in with an account on that zerver.

`BACKEND_URL` is read at build time; without it a debug build talks to `http://localhost:3000` and a release build to `https://api.zwipe.net`. The session, the last theme and the last counters live in the browser's `localStorage` for this origin.
