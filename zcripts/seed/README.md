# Seed

Fills a local database with accounts, decks and cards, so a branch can be tested on a dev machine before it merges. Every script here refuses a database that isn't on this machine (`guard.sh`, checked by `guard_test.sh`): the URL, every `host=` and `PGHOST` must be a local socket or localhost, and `hostaddr=`, `service=`, a percent-encoded keyword, `PGHOSTADDR` and `PGSERVICE` are refused outright.

The scripts hand `psql` their SQL over stdin and their settings as `SET` lines, so a `psql` that runs inside a Postgres container works as well as a native one (`stdin_test.sh` checks it).

## Before you seed

1. Run the migrations: `sqlx migrate run --source zerver/migrations`.
2. Run the card sync once, so `latest_cards` holds the catalog: `DATABASE_URL=<local> RUST_LOG=info cargo run --release --bin zervice`. It reads Scryfall's public bulk data and takes several minutes.

## Seeding

```bash
DATABASE_URL=postgres://zwipe@localhost:5432/zwipe zcripts/seed/seed.sh small
```

- `small`: the three test accounts and 40 users, enough for any UI check.
- `load`: the three test accounts and 100,000 users, for finding what breaks well past production's size.
- `--reset` removes every seeded account first; `reset` alone removes them and stops. A run without it refuses while seeded accounts exist.
- A whole number at the end picks a different world; the same number always gives the same rows. The default is 42. Dates count back from the time of the run, so accounts and decks always look recent.

On a ThinkPad, `small` takes about a second. `load` makes about 350,000 decks and 22.6 million deck cards in about 10 minutes and 4.5 GB, and resetting it takes about 25 minutes, so a database meant for repeated load runs is quicker to drop and rebuild. `/tmp` on a tmpfs is too small for `load`.

Most users have 2 or 3 decks, some 4 to 7, a few 8 to 20. Each deck has a commander from the 1,500 most played, 30 to 70 cards from that commander's colors (most played first), basic lands, up to 15 maybeboard and 10 sideboard cards, up to three stars and a few flags. About one deck in ten has a share link, and each user saves up to five commanders for later.

## The accounts

Every seeded account's email ends in `@seed.zwipe.test`, and every one has the password `Seedling.Local.42`. That password is made up for seeded local databases and exists nowhere else.

- `seed_tester`: 12 decks.
- `seed_newbie`: no decks.
- `seed_unverified`: email not verified, two small decks under the lower card limit.
- `seed_000001` and on: the numbered users.

`activity.sh` gives one account a busy year of Activity; see the top of `activity.sql`.
