# "invalid oracle id" on add card

**Status: SHIPPED 2026-09-22. Migration `20260922120000` and the ingest fix are
live on prod: zero null oracle ids, zero reversible printings in the search
view. Fixed without a client release, so installs already in the field stopped
failing the moment it landed. Outcome recorded in the progress log; this file
stays for the diagnosis and the measurements behind the sort-key choice.**

**One sentence:** a card whose `oracle_id` is absent is sent to the server as
an empty string, which the server rejects with a 422 the user sees as a
failed add.

## Root cause, confirmed

`HttpCreateDeckCard::new` (`zwipe-core/src/http/contracts/deck_card.rs`):

```rust
oracle_id: scryfall_data
    .oracle_id
    .map(|id| id.to_string())
    .unwrap_or_default(),   // None becomes ""
```

`ScryfallData::oracle_id` is `Option<Uuid>`; the wire field is a `String`. So
`None` ships as `""`, the server parses it with `Uuid::parse_str`, and
`create_deck_card.rs:59` turns the failure into
`UnprocessableEntity("invalid oracle id: ...")`, the exact message in the
reports.

Both reported actions go through this one constructor: `add.rs:398` and
`add.rs:421` (add_card) and `quick_add.rs:163` (quick_add_card). That matches
the 5/1 split in the error table. `remove.rs:439` and `view.rs:976` use it too,
so the same failure is reachable from the remove screen and the card list.

Note the error names the *oracle* id, not the card id, so the printing id
parsed fine. Only the oracle id was missing.

## Which cards: answered (local catalog, 117,631 rows, 2026-09-22)

**81 cards, every one of layout `reversible_card`.** No other layout is
affected.

```
 layout          | count
-----------------+-------
 reversible_card |    81
```

Three things the census settled:

1. **They are served to users.** All 81 are in `latest_cards`, so they appear
   in search and are swipeable. This is not a hidden corner of the catalog.
2. **They are cards people want.** The sample includes Hallowed Fountain,
   Blood Crypt, Anointed Procession, Anje Falkenrath and Jinnie Fay: shock
   lands and Commander staples. That is why real installs hit it, and why six
   reports understates the real rate.
3. **The correct value is already stored.** Every one of the 81 has
   `card_faces->0->>'oracle_id'` populated, and in **all 81 cases both faces
   carry the identical oracle id** (81 same, 0 differ, 0 null). A reversible
   card is one card printed on both sides, so there is no "which face" policy
   question to settle. There is exactly one right answer sitting in the row.

Re-run the census against prod before applying anything; the local catalog
may lag.

## The fix: three parts, no client release

Prod census 2026-09-22 (via the owner): **82** rows, all `reversible_card`, all
82 in `latest_cards`, all 82 with both faces carrying the same id, none null.
Local showed 81; prod simply has one more card.

Because the correct value is already in the row, this is repairable
server-side, which fixes **every client including shipped 1.10.1** with no
release. But the backfill alone is not safe on its own, for the reason below.

### 1. Deprioritize reversible printings in `latest_cards` (do this first)

`latest_cards` is `DISTINCT ON (COALESCE(oracle_id, id))`. Today the affected
rows have no oracle_id, so each is its own group and survives alone. Give them
an oracle_id and they merge into the group that already holds every normal
printing of that card, and the `ORDER BY` picks one winner.

Measured on the local catalog: of the 71 resulting groups, **31 would be won
by the reversible printing**, whose `name` is the doubled form ("Hallowed
Fountain // Hallowed Fountain"). Backfilling alone would therefore fix the add
error and hand 31 popular cards a malformed display name.

Add `(sd.layout = 'reversible_card')` to the `ORDER BY`, in the same
deprioritizing spirit as the existing `promo` and `oversized` keys. With it,
**all 71 groups pick a normal printing and zero reversible printings win**;
every affected card has a normal alternative. New migration, following the
three that already rewrite this view.

### 2. Backfill the ids

```sql
UPDATE scryfall_data
   SET oracle_id = (card_faces->0->>'oracle_id')::uuid
 WHERE oracle_id IS NULL
   AND card_faces->0->>'oracle_id' IS NOT NULL;
```

Tested inside a transaction against the local catalog on 2026-09-22: 81 rows
updated, zero left null, rolled back. Nothing user-visible changes until
`latest_cards` is refreshed, so the migration in step 1 can land in the same
push.

### 3. Stop it regressing at ingest

The ingest path carries Scryfall's `oracle_id` straight through as
`Option<Uuid>` (`zwipe-core/.../scryfall_data/mod.rs:79` into
`outbound/sqlx/card/models.rs`), so lift the value from the faces when the top
level is absent. Without this the next `zervice` run reintroduces all 82.

## Second defect, same root, silent

`add.rs:1104` builds a local `DeckCard` with
`oracle_id: oracle_id.unwrap_or_default()`, which is the nil UUID rather than
an empty string. No error is raised; the undo path just puts
`00000000-0000-0000-0000-000000000000` into `mb_entries` local state. Fix it
in the same pass.

(`validate_deck.rs:626` has the same call but is a test fixture. Leave it.)

## Investigation: find out which cards, without shipping anything

The reports tell us the failure happened but not what failed. `client_errors`
carries only the server's message ("invalid oracle id: ..."), so six
occurrences identify zero cards. Close that first; the fixes are easier to
choose once we know the population.

**1. Log the printing id server-side (highest leverage, no client release).**
The 422 path still has a *valid* `scryfall_data_id`: only the oracle id was
unparseable. `create_deck_card.rs` currently discards it, because

```rust
let request = CreateDeckCard::new(user.id, &deck_id, &body.scryfall_data_id,
                                  &body.oracle_id, ...)?;   // `?` drops the body
```

Replace the bare `?` with a `map_err` that logs `body.scryfall_data_id` (and
whether `body.oracle_id` was empty versus merely malformed) before converting.
One deploy, and every subsequent occurrence names its card. Do the same in the
three sibling handlers that emit this message: `get_printings.rs:15`,
`commander_maybeboard.rs:36`, `skip_deck_card.rs:32`.

Then resolve any captured id with:

```sql
SELECT id, name, layout, oracle_id FROM scryfall_data WHERE id = '<captured id>';
```

**2. Census the catalog (answers it immediately if the hypothesis holds).**

```sql
SELECT layout, count(*) FROM latest_cards WHERE oracle_id IS NULL GROUP BY layout;
SELECT name, layout FROM latest_cards WHERE oracle_id IS NULL LIMIT 20;
-- and the wider table, since latest_cards dedupes per oracle_id and may hide them
SELECT layout, count(*) FROM scryfall_data WHERE oracle_id IS NULL GROUP BY layout;
```

If the first query returns nothing but the third does, the matview is already
filtering them and the failing cards reach the client by some other path,
which would be worth knowing on its own.

**3. Check what the sync stores.** Whether `zervice` preserves a NULL
top-level `oracle_id` or could lift it from the faces at ingest. If it can be
fixed at ingest, options 2 and 3 below both become unnecessary.

**4. Reproduce locally.** Once a name is known, add that card to a deck
against a dev server. A failing repro also gives the regression test a real
subject instead of a synthetic `None`.

**5. Client-side breadcrumb (only if the above is not enough).** Report an
error at construction when `oracle_id` is `None`, carrying name and layout, so
the card is named without a round trip. Costs a client release, so treat it as
the fallback rather than the first move.

## Fix options

1. **Stop sending a broken value.** Make construction fallible
   (`HttpCreateDeckCard::try_new` returning `Result`) so the app shows a real
   message instead of a server 422. No wire change, smallest, but the user
   still cannot add the card.
2. **Fall back to the first face's oracle id.** Fixes reversible cards
   properly, but a reversible card genuinely has one oracle id per face, so
   picking face 0 is a policy decision, not an obvious default.
3. **Keep such cards out of the pool.** If the query shows a small, clearly
   broken set, excluding them from `latest_cards` at sync time means they never
   reach the swipe stack at all.
4. **Make it unrepresentable.** Wire field becomes a real `Uuid`. Correct, and
   a contract change, so it needs the forced-update dance in
   `synergy_flag_into_body.md`. Only worth bundling with that one.

Recommended order: run the query, then do (1) plus the `add.rs:1104` fix as
the immediate stop-the-bleeding, then pick between (2) and (3) based on what
the query shows.

## Verification

- Reproduce first: build a `ScryfallData` with `oracle_id: None` and assert
  `HttpCreateDeckCard::new` currently yields `""`. That test is the regression
  guard whichever fix is chosen.
- After: adding an affected card either succeeds or fails with a message that
  names the real problem, and nothing reaches the server as `""`.
- Re-read `client_errors` at the next sweep (due by 2026-10-06) to confirm the
  count stops growing.
