# Commander maybeboard

**Status: BUILD-READY 2026-08-13** (owner direction 2026-08-12; grew out of
the "commander shortlist" feature request of 2026-07-11 — this file was
`commander_shortlist.md`). All three owner decisions resolved 2026-08-13 at
the proposals: entry point = Decks action bar "Commanders" button,
keep-entry-after-create = keep, cap = 50.

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

- **Adding (revised 2026-08-13 during owner review — three paths, not one):**
  1. **Up-swipe during any commander Zwipe-select** (the command-zone swipe
     flow in create/edit deck): up-swipe = maybe. Toast ("Added to your
     commander maybeboard"), the pile advances, selection continues — saving
     is non-committal and never ends the flow.
  2. **The screen's own Swipe overlay** (a "Swipe" action-bar button mounts
     the same SwipeSelect in Commander mode): up saves as above;
     **right-swipe seeds a NEW deck** with that commander (create screen,
     format Commander preselected) — the maybeboard doubles as a
     commander-discovery surface, not just an archive.
  3. **Quick add**: the screen's input runs a debounced commander name search
     (top 8, shared floating chips hung downward — the bar sits at the top of
     the page); tapping a chip saves it. Pure quick add, deliberately NOT a
     list filter (owner + assistant concurred 2026-08-13: one input driving
     two result sets reads ambiguous, and the 50-cap list never needs name
     narrowing — the Show pips cover it, deck-list grammar). Already-saved
     commanders drop out of the chips.
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
- **Entry point (decided 2026-08-13, revised same day):** a "Commander
  maybeboard" button inside a new More bottom sheet on the Decks screen
  (action bar Back · Create · More, More right-most) — owner call: it's a
  side feature and shouldn't crowd the action bar. (Earlier candidates: a
  dedicated action-bar "Commanders" button, a Home tile.)
- **After "Create deck with this commander" (decided 2026-08-13):** the entry
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
- **Cap: 50 per user (decided 2026-08-13)** — enforced in the service inside
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

## Serving split (decided 2026-08-13)

The maybeboard's own Swipe overlay **excludes already-saved commanders** from
its pile (discovery mode — a saved commander is a wasted deal); deck
create/edit commander select **keeps serving them** (decision mode — those
are the ones you're there to pick). Client-side page filter in SwipeSelect
(`exclude_oracle_ids` prop, only the maybeboard passes it), with exhaustion
keyed on the pre-filter page and bounded continue-pulling so a fully-excluded
page can't dead-end the pile.

**Stack-position caching: rejected** (owner + assistant concurred). The pile
order reshuffles daily (seed `{user_id}:{date}`) so a cached cursor stales
within 24h, and the overlay has too many host identities (per-deck
create/edit x modes, keyless maybeboard) for the per-deck park pattern. The
exclusion split above delivers the actual value (fresh cards on reopen) for
near-zero machinery.

## Out of scope v1

Snow/partner special-casing (partners save individually), ordering beyond
recency, notes on entries, sharing the maybeboard, surfacing it inside deck
creation as a "start from maybeboard" picker (natural phase 2).
