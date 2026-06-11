# Deck Import (Moxfield / Archidekt)

Import existing decks so users don't rebuild by hand. High-priority for adoption
(see `status/backlog.md`). This plan covers **Archidekt first** (open API, fully
proven) and **Moxfield second** (ToS-gated, needs authorization).

Status: **built on `feat/deck-import-archidekt`** (2026-06-10). Shipped shape is
simpler than the original plan below: `POST /api/deck/{deck_id}/import/archidekt`
imports an Archidekt deck's cards **into an existing deck**, exactly like the
plain-text importer — same board selection, same add/replace modes, same
`ImportDeckCardsResult` response. It does **not** create a deck and does **not**
set commander/format (deliberately dropped for simplicity; the format-id table
below is kept for a future opt-in sync). Resolution is by Scryfall printing UID
with a name fallback for null-oracle reversible printings. Both importers gained
a `replace` flag: the imported board is made to exactly match the list (other
boards untouched; an empty import never wipes). The iOS import screen has
From (Text/Archidekt), Mode (Add/Replace), and Board chip rows.

Earlier prototype: `zcripts/deck-import/archidekt_probe.sh` fetches a public
Archidekt deck, parses it to Zwipe's shape, and resolves every printing against
the local `scryfall_data` table.

---

## Why Archidekt is easy and Moxfield is not

| | Archidekt | Moxfield |
|---|---|---|
| Endpoint | `GET https://archidekt.com/api/decks/{id}/` | `GET https://api.moxfield.com/v2/decks/all/{publicId}` |
| Auth | none, open public read | Cloudflare + **ToS forbids scraping** |
| Access | just set a descriptive User-Agent | must email support@moxfield.com for an authorized custom User-Agent |
| Deck id | numeric, `/decks/(\d+)` | alphanumeric, `/decks/([a-zA-Z0-9-_]+)` |
| Docs | none (open beta, shape may drift) | none (internal API) |

**Decision:** ship Archidekt URL import now; for Moxfield, ship the existing
plain-text paste path (works today, zero ToS exposure) and pursue an authorized
User-Agent in parallel before adding a Moxfield URL flow.

---

## The key finding: resolve by Scryfall UID, not by name

Archidekt embeds the **Scryfall printing UUID** on every card at `card.uid`
(and the oracle_id at `card.oracleCard.uid`). `scryfall_data.id` IS that UUID.
So import resolves with a **direct batch UID lookup** — no fuzzy name matching,
exact printing preserved. This is strictly better than the current text importer,
which resolves by lowercased name and collapses to a default printing.

Proven on two real decks (local DB, kept current by zervice):

```
shorikai  (meld + MDFC): 84 printings, 84 resolved, 0 misses, commander detected
satya     (normal):      83 printings, 83 resolved, 0 misses, commander detected
```

Meld pieces, modal-DFC, and commander tagging all resolve with no special-casing.

### Archidekt JSON field map
- name → `card.oracleCard.name`
- printing UUID → `card.uid`  → **`scryfall_data.id`** (primary resolution key)
- oracle_id → `card.oracleCard.uid` → `scryfall_data.oracle_id` (fallback / dedup)
- quantity → `quantity`
- set code → `card.edition.editioncode`
- collector number → `card.collectorNumber`
- layout → `card.oracleCard.layout` (`normal`/`modal_dfc`/`meld`/…)
- per-card tags → `categories` (array of names)
- deck format → top-level `deckFormat` (int; **3 = Commander**; map TBD)

### Command zone & board detection
- Deck-level `categories[]` carry two flags:
  - `isPremier == true` → that category name is the **command zone** (e.g. "Commander").
  - `includedInDeck == false` → maybeboard/sideboard-style (e.g. "Attraction").
- A card is **in the command zone** if its `categories` intersects a premier name.
- A card is **excluded from the deck** if its `categories` carries an excluded name
  (route to maybeboard, or skip in v1).

---

## Server-side endpoint (the right home for this)

Keep the iOS client dumb: it sends a URL, the server does all fetching/parsing.
Per the patch-discipline doctrine (`backlog.md`), Archidekt's undocumented shape
can drift, but a **server** parser is patchable in minutes; an app-baked parser
would need an App Store cycle. So fetch + parse live in zerver.

```
POST /api/deck/import/archidekt   (private, authenticated, rate-limited)
body: { "url": "https://archidekt.com/decks/13769484/shorikai" }
resp: { "deck_id": "...", "imported": 99, "unresolved": [ {name, reason}, ... ] }
```

Flow:
1. Extract numeric id (`/decks/(\d+)`); reject otherwise.
2. Fetch `https://archidekt.com/api/decks/{id}/` with UA
   `ZwipeTCG/1.0 (+https://zwipe.net)`, a timeout, and a body-size cap.
   Map non-200 → clear errors (404 private/unlisted, etc).
3. Parse to `(uid, oracle_uid, quantity, zone, name)` rows (logic proven in the
   probe script).
4. Resolve: batch `get_multiple_scryfall_data(uids)` (**already exists** in the
   card repo). Misses → fall back to `find_cards_by_exact_names` (**already
   exists**); still-missing → `unresolved[]`.
5. Create a **new** deck owned by the user (name from `deck.name`, format mapped
   from `deckFormat`). Insert `deck_cards` with the exact printing. Set command
   zone: 1 premier card → `commander`; 2 → `commander` + `partner_commander`.
   (Background / signature spell / companion: defer to phase 3.)
6. Enforce existing limits (250 cards/deck, deck-count cap, unverified soft caps)
   — reuse the guards in the deck-card service.
7. Return deck id + unresolved report.

Note: this **creates** a deck, so it's a new service method, not the existing
`import_deck_cards` (which imports text lines INTO an existing deck). It reuses
that path's *resolver primitives* and limit guards, not its entry point.

---

## Client UX (iOS / zwiper)
- Deck list → "Import deck" → paste an Archidekt URL → POST → navigate to the new
  deck; toast the unresolved count if any.
- Client sends only the URL. All parsing/resolution server-side → patchable
  without an app release.

---

## Phasing
1. **Archidekt URL import** — endpoint + iOS paste-URL UI. UID resolution,
   command zone, unresolved report. (Data says ~100% resolution.)
2. **Moxfield** — email support@moxfield.com for an authorized User-Agent; add
   `POST /api/deck/import/moxfield` reusing the same resolver. Until authorized,
   point Moxfield users at the existing plain-text paste import.
3. **Polish** — `deckFormat` int→`Format` mapping table; partner/background/
   signature-spell/companion command-zone mapping; route excluded categories to
   the maybeboard; cache fetches by `deck_id`+`updatedAt`; contract test against a
   pinned known deck to catch Archidekt shape drift.

---

## Archidekt `deckFormat` id table (verified 2026-06-10)

**Not currently used by code** — the shipped importer takes only the card list.
Kept verified here for a future opt-in commander/format sync.

Archidekt's format ids do **not** follow their documented ordering (18 is Alchemy,
not Premodern; 13 is Standard Brawl, not Brawl; 21 is Gladiator), so each was
verified against a real deck.

| id | Archidekt format | Zwipe format |
|----|------------------|--------------|
| 1 | Standard | standard |
| 2 | Modern | modern |
| 3 | Commander | commander |
| 4 | Legacy | legacy |
| 5 | Vintage | vintage |
| 6 | Pauper | pauper |
| 7 | Custom | — (None) |
| 8 | Frontier | — (None) |
| 9 | Future Standard | future |
| 10 | Penny Dreadful | penny |
| 11 | 1v1 Commander | commander (closest match) |
| 12 | Duel Commander | duel |
| 13 | Standard Brawl | standardbrawl |
| 14 | Oathbreaker | oathbreaker |
| 15 | Pioneer | pioneer |
| 16 | Historic | historic |
| 17 | Pauper EDH | paupercommander |
| 18 | Alchemy | alchemy |
| 20 | Brawl | brawl |
| 21 | Gladiator | gladiator |
| 22 | Premodern | premodern |
| 23 | PreDH | predh |
| 24 | Timeless | timeless |

Effectively the whole Archidekt dropdown, verified one deck per format. Archidekt
has no Explorer. Ids 19 and 25+ were never seen (Old School / Historic Brawl not
located); any unmapped id → no format set, which is the desired fallback.

## Open questions / discovery
- **Two-commander / partner detection.** Premier category can hold 2 cards; confirm
  ordering and how Archidekt distinguishes partner vs background vs companion.
- **Private/unlisted decks.** Confirm the API's exact response so we surface a
  helpful "make the deck public" error.
- **Politeness.** Pick a timeout, body cap, and per-user rate limit; consider a
  short cache keyed on `deck_id`+`updatedAt`.

## Test assets
- `zcripts/deck-import/archidekt_probe.sh <url>` — fetch + parse + resolve, no DB
  writes. The parsing/resolution reference for the real endpoint.
- Known-good decks: `13769484` (meld+MDFC), `11493358` (normal). Both 100%.
