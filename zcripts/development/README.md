# Development

Everything for running zwipe on your own machine: setting the machine up, filling a local database, and running the whole stack. Every script here works on local databases only.

## What's here

| File | What it does |
|---|---|
| `setup_macos.sh`, `setup_omarchy.sh`, `setup_ubuntu.sh` | Set up a new machine, one per platform |
| `setup_macos.md`, `setup_omarchy.md`, `setup_ubuntu.md` | Each platform's guide: packages, prerequisites, gotchas |
| `reset_macos.sh`, `reset_omarchy.sh`, `reset_ubuntu.sh` | Wipe the local `zerver` database back to a clean, migrated state |
| `up.sh`, `down.sh` | Start and stop the local stack |
| `stack_common.sh` | What `up.sh` and `down.sh` share: ports, work dir, which database to copy |
| `drive.mjs`, `login.json` | Drive Chrome for the verify skill, and the sign-in steps it replays |
| `seed.sh` | Seed a local database, running `seed.sql` (the accounts, decks, cards and their swipes), `seed_activity.sql` for `seed_tester`, `seed_pools.sql` (the pooled card signal) and `seed_reset.sql` (removes them all) |
| `seed_activity.sh`, `seed_activity.sql` | Give one account a busy year of Activity |
| `guard.sh` | Refuses any database that isn't on this machine; every seed script runs it first |
| `stack_common_test.sh`, `drive_test.sh`, `guard_test.sh`, `seed_stdin_test.sh` | Tests for the scripts above; run them after changing one |

## Set up a machine

Start with the guide for your platform: [macOS](setup_macos.md), [Omarchy](setup_omarchy.md) or [Ubuntu](setup_ubuntu.md).

- `setup_<os>.sh` is for a new machine or a fresh OS install. It installs `dioxus-cli` (pinned to the `dioxus` version in `zwiper/Cargo.toml`), `sqlx-cli`, Postgres and the OS build dependencies, creates the `zerver` database, runs the migrations, provisions the `zervice` role and writes `zerver/.env` and `zwiper/.env` with localhost defaults. Safe to run again.
- `reset_<os>.sh` drops and recreates the `zerver` database, rewrites the `.env` files and runs the migrations again. It doesn't touch toolchains.

The three platforms differ only in package manager and build dependencies. macOS also sets up an iOS Simulator.

## Run the stack

```bash
zcripts/development/up.sh            # prints the URLs and the login when it's ready
zcripts/development/down.sh          # stops both and drops the copy
KEEP=1 zcripts/development/down.sh   # stops both and keeps the copy
```

`up.sh` copies your dev database, migrates and seeds the copy, then starts zerver on :3100 and the web app on :8100. Other ports: `ZERVER_PORT=3200 WEB_PORT=8200`, the same for `down.sh`. `stack_common.sh` holds what both share; `drive.mjs` drives Chrome for the verify skill (`.claude/skills/verify/SKILL.md`), with `login.json` its sign-in steps. The `*_test.sh` files check them.

## Seed a database

Before seeding, run the migrations (`sqlx migrate run --source zerver/migrations`) and the card sync once (`DATABASE_URL=<local> RUST_LOG=info cargo run --release --bin zervice`, several minutes).

```bash
DATABASE_URL=postgres://zwipe@localhost:5432/zwipe zcripts/development/seed.sh small
```

- `small`: the three test accounts and 40 users, enough for any UI check. About a second.
- `load`: the three test accounts and 100,000 users, for finding what breaks well past production's size. About 10 minutes and 4.5 GB; resetting it takes about 25, so it's quicker to drop and rebuild the database. `/tmp` on a tmpfs is too small for it.
- `--reset` removes every seeded account first; `reset` alone removes them and stops. A run without it refuses while seeded accounts exist.
- A whole number at the end picks a different world; the same number always gives the same rows. The default is 42.

Seeded emails end in `@seed.zwipe.test` and every password is `Seedling.Local.42`, made up for local databases only. The accounts are `seed_tester` (12 decks), `seed_newbie` (none), `seed_unverified` (unverified, two small decks) and `seed_000001` on. `seed_tester` gets a busy year of Activity, and the numbered accounts the swipes that built their decks (the first 2,000 at load size), all summed into the pooled tables, so the card stats and Most skipped have numbers. `seed_activity.sh` gives any other account a year too.

`guard.sh` is what refuses any database that isn't on this machine: the URL, every `host=` and `PGHOST` must be a local socket or localhost, and `hostaddr=`, `service=`, a percent-encoded keyword, `PGHOSTADDR` and `PGSERVICE` are refused outright. The seed scripts hand `psql` their SQL over stdin, so a `psql` inside a Postgres container works too.
