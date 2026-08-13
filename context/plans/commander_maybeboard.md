# Commander maybeboard

**Status: SPECCED 2026-08-13** (owner direction 2026-08-12; grew out of the
"commander shortlist" feature request of 2026-07-11 — this file was
`commander_shortlist.md`). Open decisions marked **OWNER** below; everything
else is build-ready.

## The need (unchanged from the original request)

Real and repeatedly felt: while swiping commanders you find one you want to
**consider later** without committing it to a deck. Today the only
commander-swipe flow lives inside deck creation, where "save for later" has no
home — a deck has one commander, so an up-swipe there had no meaning.

"Collecting commanders I'm interested in" and "picking THE commander for this
deck" are two different intents; this feature gives the first one its own home.
It is **per-user**, not per-deck.

## Name: "commander maybeboard"

Owner call, and the right one: up-swipe already means *maybe* everywhere the
app swipes (add screen up-swipe → the deck's maybeboard). Extending the same
gesture in commander select to mean "maybe this commander" introduces zero new
vocabulary — the muscle memory does the teaching. The per-deck vs per-user
wrinkle is handled by the qualifier in copy ("Commander maybeboard" vs a
deck's "Maybeboard"). Rejected: "shortlist" (new vocab), "saved commanders"
(generic, no gesture tie-in).

## UX

- **Adding — only during commander Zwipe-select** (the command-zone swipe flow
  in create/edit deck): **up-swipe = maybe**. Toast ("Added to your commander
  maybeboard"), the pile advances, selection continues — saving is
  non-committal and never ends the flow. No other entry points in v1.
- **The screen — "Commander maybeboard":** a list page of card entries with
  art, using the established card-row grammar (`SharedCardRow`: art thumb,
  name, expandable details, `show_classification`). Newest first.
  - **Filterable:** name input + the five color pips (closed vocab → rendered
    statically per the skeleton rule; identity contains-all semantics like
    the deck list's Show row).
  - **Per-entry actions:** **"Create deck with this commander"** → navigates
    to the new-deck profile screen with the commander pre-filled, gated on
    deck capacity (unverified 1-deck / 20-deck cap → the deck list's existing
    warning toast pattern). **Remove** (un-maybe) — one tap, no confirm.
  - **Empty state:** points at the gesture ("Up-swipe a commander while
    picking one to save it here").
- **Hints:** one-time `commander_maybeboard` key (hints_shown, no migration)
  on first screen visit; the swipe-select hint gains an up-swipe bullet.
- **OWNER — placement of the screen's entry point.** Proposal: a "Commanders"
  button on the Decks screen action bar (Back · Commanders · Create) — it's
  where deck-starting thoughts live. Alternative: a Home tile.
- **OWNER — after "Create deck with this commander":** proposal — the entry
  STAYS on the maybeboard (removal is one tap; auto-removal would surprise
  anyone building two decks around one commander).

## Data

New table, additive migration:

```sql
CREATE TABLE commander_maybeboard (
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    oracle_id  UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, oracle_id)
);
```

- **Oracle-keyed** (printing-agnostic), matching the suppressions table;
  hydrated through `latest_cards` on read, so entries always show the
  preferred printing (the 2026-08-12 English-preference fix makes this the
  right row by construction).
- **Cap: 50 per user** (OWNER may adjust) — enforced in the service inside
  the insert transaction, `MAX_CARDS_PER_DECK` pattern.

## API (auth'd, private nest)

- `GET  /api/user/commander-maybeboard` → hydrated `Vec<Card>`, newest first.
- `POST /api/user/commander-maybeboard/{oracle_id}` → idempotent OK (dup =
  no-op success), 422 over cap, 404 unknown oracle.
- `DELETE /api/user/commander-maybeboard/{oracle_id}` → idempotent OK.

Hexagonal per the house pattern: core request types + validation, zerver
ports/service/repo (`Database*` wrapper), contract types in core `http/`.

## Client

- Outbound client module (3 calls).
- Swipe-select screen: enable/route the up-swipe (verify the current
  `SwipeStack` config there — up may be disabled today) → POST + swipe
  telemetry + toast.
- New screen + route (`Router::CommanderMaybeboard`), back-gesture aware.
- Create-deck prefill: route carries the picked card (scryfall_data_id from
  the hydrated entry); the create screen seeds its commander selection and
  the existing eligibility/validation flow takes over.

## Estimate

Server ~0.5 day (migration + endpoints + tests), client ~1–1.5 days (screen +
swipe wiring + prefill), plus owner UI review cycles on the list page. Rides
its own train (a real feature → minor bump per `development/versioning.md`).

## Out of scope v1

Snow/partner special-casing (partners save individually), ordering beyond
recency, notes on entries, sharing the maybeboard, surfacing it inside deck
creation as a "start from maybeboard" picker (natural phase 2).
