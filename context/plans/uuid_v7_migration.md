# UUID v4 → v7 everywhere

**Status: HORIZON (owner sequencing recorded 2026-08-05). Not scheduled — do
when there's time. Prerequisite-ordered: Postgres 18 first, then the data
migration, then the call sites.**

**One sentence:** move every ID we mint to time-ordered UUIDv7 (RFC 9562) —
new-row generation AND a one-time rewrite of existing v4 IDs — for b-tree
insert locality and chronologically sortable keys.

## Owner's sequencing

1. **Upgrade prod + dev Postgres 16 → 18** (native `uuidv7()` lands in 18).
   Full test suite green on 18 before anything else moves. CI's
   `postgres:16` service image bumps in the same pass
   (`.github/workflows/deploy-zerver.yml`).
2. **Migration: regenerate existing IDs as v7 — everything we mint.** New
   v7 per row, all FK references updated in lockstep.
3. **Call sites: mint v7 going forward.** App side: the `Uuid::new_v4()`
   sites (~82, majority test helpers) → `Uuid::now_v7()` (uuid crate `v7`
   feature; allowed in zwipe-core). DB side: the three `DEFAULT
   gen_random_uuid()` tables (`users`, `decks`, sync metrics) → `DEFAULT
   uuidv7()`.

## Landmines the migration step must handle (the "surely it isn't that
hard" is mostly these)

- **Scryfall card IDs are EXEMPT — never regenerate.** `cards.id` (and
  oracle ids etc.) are Scryfall's own identifiers; the nightly sync matches
  on them. Rewriting them orphans the whole catalog on the next sync.
  "Everything" = every ID *we* mint, not IDs we mirror from outside.
- **FK lockstep.** One transaction per entity family: regenerate the PK,
  cascade the new value through every referencing column (deck_cards,
  refresh_tokens, signal tables, suppressions, audit/events, …). Build the
  full reference inventory from `information_schema` at write time rather
  than hand-listing — hand lists rot.
- **Deck share links break.** Public share URLs embed `deck_id`; a
  regenerated id 404s every link already posted (Reddit, Discord, texts).
  Decide before running: accept the breakage (announce it), or keep an
  `old_id → new_id` alias table the share endpoint consults. Same question
  for any other id that has ever left the system in a URL.
- **Mass logout.** Access JWTs and stored sessions carry `user_id` claims;
  regenerated user ids invalidate every live session at a stroke. Fine —
  but schedule it like the deliberate mass-logout it is (quiet hour,
  release-note line), don't let it surprise as an incident.
- **Backdate the synthesized v7s.** A v7 minted at migration time stamps
  every historical row "today", which destroys the ordering property for
  all pre-migration data. Synthesize each row's v7 from its own
  `created_at` (v7 = 48-bit ms timestamp + random tail) so old rows sort
  where they actually belong.
- **Timestamp leak — ACCEPTED (owner, explicit, 2026-08-05).** v7 ids
  embed creation time; deck ids ride public share URLs, so anyone can
  decode when a deck was made. Owner's call: fine, it's a deck-building
  app — nothing sensitive rides on when a deck was created. Not a
  revisit-later item.

## Verification sketch

- Postgres 18: full workspace suite + a prod-parity restore test before the
  prod upgrade itself.
- Migration: row counts + FK integrity (`NOT VALID` constraints validated
  after), spot-check share links (per the alias decision), old client
  session → clean re-login, nightly sync green (proves card ids untouched).
- Call sites: grep gate — no `new_v4` outside tests once the swap lands
  (test helpers may keep v4; they exercise "any valid uuid").

## Explicitly out

- The in-memory undo-entry id (stays `u64`, `global_undo.md`).
- Anything time-critical: this whole plan waits for a quiet stretch; no
  release depends on it.
