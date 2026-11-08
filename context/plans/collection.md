# Collection

A user's card collection in Zwipe: imported from ManaBox's CSV export, edited by hand, and used to search ("In my collection" on Add and search) and to show what a deck already owns. Plan only; nothing here is built.

The product side is in `context/product/premium/collection_tracking.md`: the collection and the "Cards I own" filter are free (decided 2026-06-10), and the shopping list and owned percentage are where it meets prices. This doc is the build plan.

## Open questions for Scotty

1. **Re-import (answered: both modes, Add and Replace, like deck import).** a second ManaBox import replaces the collection (the export is the whole truth) or adds on top (two binders imported one after the other)? Proposed default: Replace, with Add as the second choice in the import screen, the same Add / Replace pair deck import has.
2. **Owned means which printing? (answered: any printing counts as owned, but the app shows the printing you own.)** Proposed: a card counts as owned when any printing of it is owned (oracle id), since a deck rarely cares which Sol Ring is in the box. A printing-exact view can come later.
3. **Binders: (answered: store the binder name and give a binder filter so people can search within binders.)** keep ManaBox's binder or list name per row (a "Binder" filter later), or flatten everything into one pile? Proposed: store it, show nothing for it in phase 1.
4. **Condition, language, purchase price: (answered: store all three.)** store them (cheap, and a later "collection value" needs price and currency) or drop them? Proposed: store all three, show none in phase 1.
5. **Cards in decks: (answered: yes for now, owned means in the collection. Later, a separate feature: decks claim owned copies from a stock, so two Sol Rings cover only two decks and Sol Ring stops being suggested once the stock is used up.)** does a card in a deck still count as owned for another deck (one Sol Ring, five decks)? Proposed: yes; "owned" is the collection, not what's free. "Free to use" needs deck assignment, which is out of scope here.
6. **Where import lives: (answered: web only; the app and web both use the collection in search.)** web only first (a file picker is natural there), or the app too (share sheet or file picker)? Proposed: web first, app in phase 2.
7. **Prices first? (answered: build phases 1 to 3 now; price snapshots are a later, separate plan.)** `collection_tracking.md` sequences this after a price snapshot table, so owned percentage and the shopping list carry dollar amounts from day one. Phases 1 to 3 here don't need it; phase 4 uses the current card prices Zwipe already has. Build 1 to 3 now, or wait?
8. **Printing level from the start? (answered: yes, people don't want to see printings they don't own.)** `collection_tracking.md` starts at the oracle level, with printings as a later refinement. ManaBox rows carry the exact printing for free, so this plan stores the printing and reads ownership by oracle id: no refinement migration later, same lookups. OK to diverge?

## Would a real export change anything?

Yes, in the details, not the shape. The design below holds either way, but one real export (a few rows across a binder, with a foil, an etched card, a non-English card and a card with a purchase price) would pin down:

- the exact header spellings and order of today's export, and whether binder exports and whole-collection exports differ (one forum report says the columns change with how the collection is organized);
- the value spellings for Foil (`normal` / `foil` / `etched`?), Condition and Language (codes or names);
- the Purchase price format (decimal separator, currency as an ISO code or a symbol);
- quoting and encoding (names with commas, accents).

Without it, the parser keys on header names case-insensitively, accepts each known alias, and treats unknown values as "not given", so a surprise degrades a field instead of failing the import.

## What ManaBox exports

ManaBox has no collection share link or API; the export is CSV, for the whole collection (collection tab, top-right menu) or one binder or list.

Confirmed by ManaBox's own guide ([import and export](https://www.manabox.app/guides/collection/import-export/)): the export carries "all card properties as well as the binder/list name"; on import, the minimum is the card name plus the set name or set code (Scryfall's), and a Scryfall ID can stand in for both. Accepted import columns: Name, Set code / Set name, Quantity, Foil, Card number / Collector number, Language, Condition, Purchase price, Purchase price currency, Misprint, Altered, Scryfall ID.

Reported by users (a 2023 header row on [TappedOut](https://tappedout.net/mtg-forum/tappedout/importing-csv-from-manabox/), and converters such as [RareWell's guide](https://rarewellcards.com/guides/how-to-export-manabox-collection-csv)): `Name, Set code, Set name, Collector number, Foil, Rarity, Quantity, ManaBox ID, Scryfall ID, Purchase price, Misprint, Altered, Condition, Language, Purchase price currency`, plus a binder name column on collection exports. Not verified against a current export (see above).

The Scryfall ID is the key: it names the exact printing, so most rows resolve without guessing.

## Data model

One table, server side, private to its user:

```sql
CREATE TABLE collection_cards (
    user_id          UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    scryfall_data_id UUID NOT NULL REFERENCES scryfall_data (id) ON DELETE RESTRICT,
    oracle_id        UUID NOT NULL,
    finish           TEXT NOT NULL DEFAULT 'nonfoil',  -- nonfoil | foil | etched
    condition        TEXT,                              -- NULL: not given
    language         TEXT,                              -- NULL: not given
    quantity         INT  NOT NULL CHECK (quantity > 0),
    binder           TEXT,                              -- ManaBox binder or list name
    purchase_price   NUMERIC(12, 2),
    purchase_currency TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, scryfall_data_id, finish, condition, language)
);
CREATE INDEX idx_collection_cards_owned ON collection_cards (user_id, oracle_id);
```

- **Printings and cards:** a row is a printing (`scryfall_data_id`), the same key `deck_cards` uses, with `oracle_id` stored beside it as `deck_cards` does, so "owned" by card is one indexed lookup. No foreign key to decks: a deck and a collection only meet by oracle id at query time.
- **Primary key:** condition and language are nullable, so the key is a unique index with `NULLS NOT DISTINCT` (the server runs Postgres 18) in place of the primary key above.
- **Non-English printings:** the nightly sync holds one printing per Scryfall id it imports. If a row's Scryfall ID is not in `scryfall_data` (a non-English printing, for one), the importer falls back to the same set and collector number, keeps the row's language, and says so in the summary.
- **Quantity 0** deletes the row; the table never holds zeros.
- **Domain types** in zwipe-core: `CollectionEntry`, `Finish` (closed enum, wire strings above), `CardCondition` (closed enum with ManaBox's spellings as aliases), all with their validation and tests there; the `Database*` wrapper stays in zerver's sqlx layer, per the purity rules.

## Adding cards

### CSV import (phase 1)

- **Endpoint:** `POST /api/collection/import`, authed, body `HttpImportCollection { csv: String, mode: ImportMode }` (`mode` defaulted to Replace per question 1, or Add if Scotty prefers; deck import's `ImportMode` already exists). Size-capped like deck import.
- **Parsing:** server side, in zerver, with the `csv` crate, so the app and web share one parser and zwipe-core stays free of it. Headers match case-insensitively with ManaBox's aliases (`Set code` or `Set name`, `Collector number` or `Card number`, `Purchase price currency` or `Purchase currency`).
- **Resolution, per row, first hit wins:** Scryfall ID; set code plus collector number; name plus set; name alone only when one printing exists (otherwise unmatched, never a guess). Rows for the same key merge their quantities.
- **Reply:** `HttpCollectionImportSummary { added, updated, removed, unmatched: Vec<UnmatchedRow { line, name, set, reason }> }`, so the screen can list what didn't land and why.
- **Atomic:** Replace runs in one transaction: a failed import leaves the old collection as it was.
- **Web:** a Collection screen with an Import button (file picker, `.csv`), the Add / Replace choice, then the summary with the unmatched list. Copy through human-copy.
- **Pasted lists:** the same screen takes a pasted text list (`4 Lightning Bolt (M10) 146`) through the deck text importer's parser, for users who don't use ManaBox.

### Manual add, remove, quantity (phase 2)

- **Endpoints:** `PUT /api/collection/cards` with `{ scryfall_data_id, finish, quantity }` (quantity 0 removes) and `GET /api/collection` (paged, searchable by name, sorted by name).
- **Clients:** the Collection screen lists rows with quantity steppers and the same card details the deck rows open; card details elsewhere (Add, the deck page) get "In collection: N" with a stepper.
- **Export:** `GET /api/collection/export` returns CSV in ManaBox's own column names, so the collection can go back.

## Searching from the collection

### "In my collection" filter (phase 3)

- **Request field:** `CardQueryBuilder` gains `owned_only: bool`, `#[serde(default)]` (false: the search as it ships today). Additive per `api_evolution.md`: an old client omits it, an old server ignores it.
- **Server:** with `owned_only` on an authed search (`search_deck_cards`, `search_cards` when a session is present), the query adds `EXISTS (SELECT 1 FROM collection_cards cc WHERE cc.user_id = $user AND cc.oracle_id = sd.oracle_id)`. Unauthed searches ignore it.
- **Clients:** a chip in the shared filter sheet ("In my collection"), shown only to a user with a non-empty collection; on Add it sits with Synergy. The filter draft and its chip live in zwipe-ui so the app, web and the terminal client match.

### Owned counts on deck rows (phase 4)

- **Endpoint:** `GET /api/collection/owned?deck_id=…` returns `{ oracle_id: owned_quantity }` for the deck's cards. A new endpoint rather than a field on the deck reply, so the deck contract stays as it is and the counts cache on their own.
- **Clients:** a deck row shows an owned tag (`2 / 4 owned`, or a check when covered); the Budget section gains "Missing: N cards, $X" from the rows not covered, which is the shopping list.

## Phases

| Phase | Server | Clients | Wire |
|---|---|---|---|
| 1 | Table, domain types, CSV parser, import endpoint | Web: Collection screen with import and summary | New endpoint only |
| 2 | Manual edit, list, export endpoints | App and web: Collection screen, steppers, "In collection" in card details; app import | New endpoints only |
| 3 | `owned_only` in the card search | Filter sheet chip on Add and search | One additive request field, `serde(default)` |
| 4 | Owned counts per deck | Owned tags on deck rows, Missing in Budget | New endpoint only |

Server ships before clients in every phase, from the same branch, as the deploy pipeline already does.

## Later, not planned here

- Build from the collection: suggested commanders and Add ordering weighted toward owned cards.
- "Free to use": which owned copies are already in decks.
- Collection value from purchase price against current prices.
- Re-import by ManaBox ID to keep per-row edits across imports.
