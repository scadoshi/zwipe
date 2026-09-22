# UUID v4 → v7 everywhere

**Status: HORIZON, reviewed and left alone 2026-09-22 (owner: "seems like a
huge lift"). Not scheduled. Read the measurements below before reopening.**

Measured on prod 2026-09-22, because the headline justification is b-tree
insert locality and that turns out not to apply here.

Only three tables mint UUIDs database-side: `decks` (1,485 rows, 1,968
lifetime inserts), `users` (986 rows, 1,004 inserts) and `zervice_metrics`.
That is the entire population the locality argument covers. Random-order
inserts cost nothing at a few thousand rows.

The tables that are actually large are `card_oracle_tags` (236k rows, 27M
lifetime inserts), `scryfall_data` (118k, 1.1 GB) and `card_profiles`
(118k). Every one is Scryfall-owned and exempt, per the first landmine
below. So the tables that would benefit are exactly the ones that must not
be touched, and that will not change: what grows here is the mirrored
catalog, not the IDs we mint.

The costs are undiminished by any of that: every posted share link 404s or
needs a permanent alias table, all users are logged out at once, and FK
lockstep plus backdated v7 synthesis still have to be right.

**If it is reopened, split it.** Minting v7 going forward is cheap and
carries none of those costs: new rows get time-ordered IDs and
`uuid_extract_timestamp`, and mixed v4/v7 in one column is fine since
nothing in the schema or the app inspects the version. Rewriting existing
IDs buys ordering for ~2,500 historical rows, and that is the half whose
cost is real while its benefit is not.

**Prerequisite 1 is nearly done.** Prod runs PostgreSQL 18.6 and native
`uuidv7()` works (verified by calling it). Local dev moved 15 to 18.6 and CI's
service image moved to `postgres:18`, both on 2026-09-22, so prerequisite 1
is complete.

**One sentence:** move every ID we mint to time-ordered UUIDv7 (RFC 9562),
new-row generation AND a one-time rewrite of existing v4 IDs, for b-tree
insert locality and chronologically sortable keys.

## Owner's sequencing

1. **Upgrade prod + dev Postgres 16 → 18** (native `uuidv7()` lands in 18).
   Full test suite green on 18 before anything else moves. CI's
   `postgres:16` service image bumps in the same pass
   (`.github/workflows/deploy-zerver.yml`).
2. **Migration: regenerate existing IDs as v7, everything we mint.** New
   v7 per row, all FK references updated in lockstep.
3. **Call sites: mint v7 going forward.** App side: the `Uuid::new_v4()`
   sites (~82, majority test helpers) → `Uuid::now_v7()` (uuid crate `v7`
   feature; allowed in zwipe-core). DB side: the three `DEFAULT
   gen_random_uuid()` tables (`users`, `decks`, sync metrics) → `DEFAULT
   uuidv7()`.

## Landmines the migration step must handle (the "surely it isn't that
hard" is mostly these)

- **Scryfall card IDs are EXEMPT, never regenerate.** `cards.id` (and
  oracle ids etc.) are Scryfall's own identifiers; the nightly sync matches
  on them. Rewriting them orphans the whole catalog on the next sync.
  "Everything" = every ID *we* mint, not IDs we mirror from outside.
- **FK lockstep.** One transaction per entity family: regenerate the PK,
  cascade the new value through every referencing column (deck_cards,
  refresh_tokens, signal tables, suppressions, audit/events, …). Build the
  full reference inventory from `information_schema` at write time rather
  than hand-listing; hand lists rot.
- **Deck share links break.** Public share URLs embed `deck_id`; a
  regenerated id 404s every link already posted (Reddit, Discord, texts).
  Decide before running: accept the breakage (announce it), or keep an
  `old_id → new_id` alias table the share endpoint consults. Same question
  for any other id that has ever left the system in a URL.
- **Mass logout.** Access JWTs and stored sessions carry `user_id` claims;
  regenerated user ids invalidate every live session at a stroke. Fine,
  but schedule it like the deliberate mass-logout it is (quiet hour,
  release-note line), don't let it surprise as an incident.
- **Backdate the synthesized v7s.** A v7 minted at migration time stamps
  every historical row "today", which destroys the ordering property for
  all pre-migration data. Synthesize each row's v7 from its own
  `created_at` (v7 = 48-bit ms timestamp + random tail) so old rows sort
  where they actually belong.
- **Timestamp leak, ACCEPTED (owner, explicit, 2026-08-05).** v7 ids
  embed creation time; deck ids ride public share URLs, so anyone can
  decode when a deck was made. Owner's call: fine, it's a deck-building
  app; nothing sensitive rides on when a deck was created. Not a
  revisit-later item.

## Verification sketch

- Postgres 18: full workspace suite + a prod-parity restore test before the
  prod upgrade itself.
- Migration: row counts + FK integrity (`NOT VALID` constraints validated
  after), spot-check share links (per the alias decision), old client
  session → clean re-login, nightly sync green (proves card ids untouched).
- Call sites: grep gate, no `new_v4` outside tests once the swap lands
  (test helpers may keep v4; they exercise "any valid uuid").

## Explicitly out

- The in-memory undo-entry id (stays `u64`, `global_undo.md`).
- Anything time-critical: this whole plan waits for a quiet stretch; no
  release depends on it.
