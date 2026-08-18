# Content Delivery: Compiled vs Fetched

**Settled 2026-08-14** (owner + assistant, after the keyword-reminder work).
The decision matrix for where content lives — compiled into a binary, served
from zerver, or both — per surface. Written so this doesn't get re-litigated
each time a new content set appears.

## The rules

Pick a delivery mode by asking two questions about the content:

1. **What cadence does it change on?**
   - *Deploy cadence* (someone edits a file, we ship): eligible to compile.
   - *Sync/user/clock cadence* (nightly Scryfall sync, per-user, per-token,
     hourly rotation): must be fetched — a compiled copy is stale the moment
     it builds.
2. **How big and how closed is it?**
   - *Small + closed vocabulary* (keyword reminders ~330 entries, changelog):
     compile-friendly; a few dozen KB.
   - *Large or open* (otag catalog: ~4,500 tags × prose, list mutates with
     community tagging): fetch-only; compiling it taxes every user to save
     one cached request.

Then apply the surface's constraint:

- **zerver** compiles all authored content it serves — the compiled copy IS
  the serving source, and a deploy is how it updates. (Changelog consts,
  keyword table, the 17k-line authored otag description overlay that merges
  into the DB at zervice sync.)
- **zite** (SSG) prefers compiled: it rebuilds on every push (~7 min), so
  compiled content is CDN-delivered, never stale, and API-independent. It
  fetches only what varies by token/time (shared decks, otag catalog,
  featured flavor, stats).
- **zwiper** (store apps) prefers served: binaries wait on store review, so
  anything that might need correcting between trains should be servable.
  Compiled copies remain as *offline fallbacks* where absence would hurt.

## The matrix (as of 1.9.2)

| Content | zerver | zite | zwiper | Fallback in app |
|---|---|---|---|---|
| Keyword reminders | compiled (serves `/api/card/keyword-reminders`) | compiled | served, catalog-cache prefetch | compiled table (full) |
| Changelog | compiled (serves `/api/changelog`) | compiled | served at startup | compiled copy (full) |
| Otag descriptions | compiled overlay → DB at sync (serves `/api/card/oracle-tags`) | fetched (share page, lazy) | served, catalog-cache | none — reveal absent, retried per screen |
| Filter catalogs (artists, sets, keyword names, oracle words, card types) | DB-derived | n/a | served | none — open vocabularies, impossible to compile |
| Deck tags catalog | DB + authored | n/a | served (authed) | none |
| Hints, guides, UI copy, themes, format/color vocab | n/a | compiled (its own copy) | compiled | n/a — describes compiled features; never hot-patched **(owner call: hints stay compiled, period)** |
| Featured flavor, marketing stats, share payloads | DB-derived | fetched | fetched | none — clock/token/user data |

## Decisions already litigated (don't reopen without new facts)

- **Keep the app's compiled fallbacks for keywords + changelog** (2026-08-14).
  Removing them was considered for binary size: the strings are ~35KB of a
  24MB binary (0.15%), the "simplification" would force zite to grow fetch
  infrastructure it doesn't need (zite consumes the compiled copies by
  design), and the served-override already makes the compiled copy a shadow,
  not a second source of truth.
- **Don't compile otag descriptions into zite** (2026-08-14). The catalog is
  a database product (Scryfall sync + authored overlay, mutating nightly), a
  compiled snapshot goes stale between deploys, ~1MB of prose in the wasm
  taxes every page for one lazy CF-cached request, and the share page is
  API-dependent anyway (the deck itself is a fetch) so there's no resilience
  win.
- **Serve keyword reminders** (2026-08-14, the trigger for this doc). Every
  new set brings new mechanics; before this, definition fixes waited on a
  store train. Now `/api/card/keyword-reminders` maps every DB keyword
  through the core table on request — a sweep lands on deploy. The map is
  keyed lowercase (the catalog query normalizes); the chips lowercase their
  lookups to match.

## How the app's fallbacks behave

The served copies are strict overrides, never dependencies: fetch fails →
compiled copy, silently (a missing catalog isn't a toast-worthy error). A
name missing from a served map → compiled entry for that name. Failed
fetches retry on next launch (keyword reminders, changelog: prefetched once)
or on next consumer screen (`ensure_*` catalogs like otags). Consequence:
success and fallback are visually identical when the copies agree — verify
the served path via the startup `GET` log line, or by shipping a server-side
text tweak and watching it appear without an app rebuild.
