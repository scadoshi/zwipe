---
name: verify
description: Boot zwipe on this machine (a throwaway copy of the local dev database, migrated and seeded, zerver and the web app) and check a change in Chrome at 1440 and 390, or on the Android emulator, then tear it down. Workers run it before reporting done; Surfer runs it on a branch. Local databases only.
---

# Verify

Run the cheap checks first: fmt, clippy and the crate tests. This skill comes after them and before QA. It answers one question: does the change work in the running app, by the steps the issue gives?

## Up

```bash
.claude/skills/verify/scripts/up.sh
```

| Step | Time |
|---|---|
| Starts Postgres if it isn't answering | ~5s |
| Copies the dev database to `zwipe_verify` (cards included, so no card sync) | 10–60s |
| Runs the migrations on the copy | ~2s |
| Runs `zcripts/seed/seed.sh small` when it exists (thinker's seed); otherwise says so and keeps the copy's data | seconds |
| Builds and starts zerver on :3100 | 3–5 min cold, ~30s warm |
| Registers `localcheck` / `Verify-local-pass-1!`, verified | ~1s |
| Builds and serves the web app on :8100 | 3–5 min cold, ~1.5 min warm |

It ends by printing the URLs, the login and where the logs are (`/tmp/zwipe-verify`).

It needs Postgres, `sqlx` (the CLI), `dx`, `node` and Chrome, which the dev setup in `zcripts/dev-env/` installs.

It refuses a database that isn't on this machine, as `zcripts/seed/activity.sh` does. The template is `VERIFY_TEMPLATE_URL`, else `zerver/.env`'s `DATABASE_URL`, else `postgres:///zerver`.

Postgres can't copy a database while anything is connected to it. If your own zerver or a psql session is open on the dev database, up.sh names them and stops. Close them and run it again.

Other ports: `ZERVER_PORT=3200 WEB_PORT=8200 up.sh`, and the same variables for down.sh.

## Check it

Use the issue's before steps, from a clean start, the same number of times as the before.

To look and click: open http://localhost:8100/login with Claude in Chrome. Size the window to 1440×900, then to 390×844, and check both.

To repeat runs and keep the evidence, use the headless driver. It prints what each labeled step reads and saves screenshots:

```bash
S=.claude/skills/verify/scripts
node $S/drive.mjs http://localhost:8100/login 1440 steps.json
H=844 node $S/drive.mjs http://localhost:8100/login 390 steps.json
```

The `localcheck` account starts empty. Until the seed lands, make what your check needs through the app or the API, or sign in as your own account, which the copy carries over from the dev database.

Start `steps.json` with the login steps from `scripts/login.json`, then add yours. The step types are listed at the top of `drive.mjs`. Read state from the page (`eval` with a `label`) or from the database (`psql` on `zwipe_verify`), not only from screenshots. Run the same file on the old build and the new build for the before and after.

For app work, build a debug APK with `BACKEND_URL=http://10.0.2.2:3100` and run it on your own emulator, never Droid's `emulator-5554`. The emulator reaches the Mac at 10.0.2.2. A debug build needs `android:usesCleartextTraffic="true"` in its manifest to talk to plain http; patch the generated manifest, never the checked-in one. A build takes 2–5 min.

## Down

```bash
.claude/skills/verify/scripts/down.sh          # stops both servers, drops zwipe_verify
KEEP=1 .claude/skills/verify/scripts/down.sh   # keeps the database to look at
```

## Report

In the done report, say what you ran it on (the sha), at which widths or on which device, the steps, and what each run read: before and after, same steps. If you couldn't run part of it, say which part.
