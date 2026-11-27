---
name: verify
description: Boot zwipe on this machine (a throwaway copy of the local dev database, migrated and seeded, zerver and the web app) and check a change in Chrome at 1440 and 390, or on the Android emulator, then tear it down. Workers run it before reporting done; Surfer runs it on a branch. Local databases only.
---

# Verify

Run the cheap checks first: fmt, clippy and the crate tests. This skill comes after them and before QA. It answers one question: does the change work in the running app, by the steps the issue gives?

## Up

```bash
zcripts/local/up.sh
```

| Step | Time |
|---|---|
| Starts Postgres if it isn't answering | ~5s |
| Copies the dev database to `zwipe_verify_3100` (cards included, so no card sync) | 10–60s |
| Runs the migrations on the copy | ~2s |
| Seeds it: `zcripts/seed/seed.sh small --reset` (the test accounts and 40 users) | ~1s |
| Builds and starts zerver on :3100 | 3–5 min cold, ~30s warm |
| Registers `localcheck` / `Verify-local-pass-1!`, verified | ~1s |
| Builds and serves the web app on :8100 | 3–5 min cold, ~1.5 min warm |

It ends by printing the URLs, the login and where the logs are (`/tmp/zwipe-verify-3100`). On a Linux box (scotland, an x86 server) the same run took 6 min 14 s cold: about 6 s to copy, migrate and seed, 2 min for zerver and 4 min for the web build.

It needs Postgres, `sqlx` (the CLI), `dx`, Node 22 or later (the driver uses its built-in WebSocket) and Chrome or Chromium. The macOS dev setup in `zcripts/dev-env/` installs them; the Linux one installs Postgres and `sqlx` but not Node or a browser, so add those yourself. On Linux the driver finds Chrome or Chromium on the PATH, or the Chromium Playwright downloads, and `CHROME` overrides it. It builds on thinker's seed (`feat/seed`): `zcripts/seed/seed.sh`, and the guard in `zcripts/seed/guard.sh`, which refuses any database that isn't on this machine. Until `feat/seed` merges, this skill lives on a branch stacked on it. The template is `VERIFY_TEMPLATE_URL`, else `zerver/.env`'s `DATABASE_URL`, else `postgres:///zerver`.

Postgres can't copy a database while anything is connected to it. If your own zerver or a psql session is open on the dev database, up.sh names them and stops. Close them and run it again. An editor's rust-analyzer holds connections too, from expanding sqlx's `query!` macros; they show up unnamed, and `pkill -f rust-analyzer-proc-macro-srv` closes them (rust-analyzer starts it again when it needs it).

A dev database that ran migrations from another branch carries tables this branch has never seen, so a check could pass against a schema the branch won't ship with. up.sh refuses that copy and names the migrations; rebuild the dev database from your branch, or point `VERIFY_TEMPLATE_URL` at one that matches, and run it again.

On Linux the driver runs Chrome without its sandbox, so it opens only localhost and 127.0.0.1, and refuses any other URL.

Other ports: `ZERVER_PORT=3200 WEB_PORT=8200 up.sh`, and the same variables for down.sh.

A run's work dir and database carry its zerver port (`/tmp/zwipe-verify-<port>`, `zwipe_verify_<port>`), so two agents on one machine, each on its own ports, never share either: one's `down.sh` stops only its own zerver, and only when the pid it saved is still a zerver. `up.sh` saves the template it copied from in the work dir, so `down.sh` drops the copy on the same Postgres without `VERIFY_TEMPLATE_URL`. `VERIFY_DIR` and `VERIFY_DB` override both.

## Check it

Use the issue's before steps, from a clean start, the same number of times as the before.

To look and click: open http://localhost:8100/login with Claude in Chrome. Before signing in there by hand, confirm the page's API is this machine: DevTools' Network tab shows its `/api/` calls going to localhost, or `performance.getEntriesByType('resource').map((e) => e.name).filter((n) => n.includes('/api/'))` in the console lists only localhost URLs; an empty list means the page hasn't called its API yet, so wait for one. Nothing gates a manual sign-in. Size the window to 1440×900, then to 390×844, and check both.

To repeat runs and keep the evidence, use the headless driver. It prints what each labeled step reads and saves screenshots:

```bash
S=zcripts/local
node $S/drive.mjs http://localhost:8100/login 1440 steps.json
H=844 node $S/drive.mjs http://localhost:8100/login 390 steps.json
```

Two kinds of account are ready. The seeded ones share the password `Seedling.Local.42`: `seed_tester` has 12 decks, `seed_newbie` none, and `seed_unverified` hasn't verified its email. `localcheck` is the empty one up.sh makes. Your own account comes over from the dev database too.

Start `steps.json` with the login steps from `scripts/login.json` (they sign in as `localcheck`; change the name and password for another account), then add yours. Under 900 wide the driver emulates a phone: touch, no hover and a coarse pointer, so 390 is phone web, not a narrow desktop window. The step types are listed at the top of `drive.mjs`. A `type` step stops the run (exit 2) unless the page has already called its API and every call went to localhost, so a sign-in never reaches a real server; `up.sh` likewise refuses a web build whose `BACKEND_URL` isn't this machine. The guards' checks are `scripts/common_test.sh`, `scripts/drive_test.sh` and `zcripts/seed/guard_test.sh`. Read state from the page (`eval` with a `label`) or from the database (`psql` on `zwipe_verify_3100`), not only from screenshots. Run the same file on the old build and the new build for the before and after.

For app work, build a debug APK with `BACKEND_URL=http://10.0.2.2:3100` and run it on your own emulator, never Droid's `emulator-5554`. The emulator reaches the Mac at 10.0.2.2. A debug build needs `android:usesCleartextTraffic="true"` in its manifest to talk to plain http; patch the generated manifest, never the checked-in one. A build takes 2–5 min.

## Down

```bash
zcripts/local/down.sh          # stops both servers, drops zwipe_verify_3100
KEEP=1 zcripts/local/down.sh   # keeps the database to look at
```

## Report

In the done report, say what you ran it on (the sha), at which widths or on which device, the steps, and what each run read: before and after, same steps. If you couldn't run part of it, say which part.
