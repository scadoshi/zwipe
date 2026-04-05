# Oracle ID Audit — Plan

Ensure all card identity **comparisons** use `oracle_id` while keeping `scryfall_data_id` as the stored value (because the user's chosen printing matters).

## The Problem

`deck_profiles` correctly stores `commander_id`, `partner_commander_id`, `background_id`, `signature_spell_id` as `scryfall_data_id` values — the user picked a specific printing. But every comparison downstream uses `scryfall_data.id` directly, which breaks when the resolved card is a different printing of the same card.

**Example:** User sets commander via printing A. Import resolves the same card name to printing B. `commander_id == request.scryfall_data_id` → false → commander gets added as a regular deck card.

The import flow already has a band-aid (services.rs:295-316) that resolves commander scryfall_data_ids → oracle_ids via an extra DB query. This pattern should be the norm, not a workaround.

## Principle

- **Store:** `scryfall_data_id` (preserves printing choice)
- **Compare:** resolve to `oracle_id` first, then compare oracle_ids

## Audit Results

### RED — Comparisons that must resolve to oracle_id

#### 1. Backend create_deck_card — commander skip check
**File:** `zerver/src/lib/domain/deck/services.rs:122`
```rust
if deck_profile.commander_id == Some(request.scryfall_data_id) {
```
**Fix:** Look up the oracle_id of `deck_profile.commander_id` and compare against `request.oracle_id` (already on CreateDeckCard). Or: resolve both to oracle_id before comparing.

#### 2. Backend get_deck — commander in entries check
**File:** `zerver/src/lib/domain/deck/services.rs:183-186`
```rust
if !entries.iter().any(|e| e.card.scryfall_data.id == commander_id) {
    let ids: ScryfallDataIds = vec![commander_id].into_iter().collect();
    self.card_repo.get_cards(&ids).await...
```
**Fix:** Resolve `commander_id` → oracle_id first. Check entries by oracle_id: `e.card.scryfall_data.oracle_id == Some(commander_oracle_id)`. Fetch still uses scryfall_data_id (to get the user's chosen printing).

#### 3. Backend get_deck — partner/background/spell fetch
**File:** `zerver/src/lib/domain/deck/services.rs:297-301`
Same pattern — these IDs are used to look up and skip additional cards. The skip check on line 338 (`additional_oracle_ids.contains(&oracle_id)`) is actually correct already because of the band-aid resolution. But the resolution itself (lines 297-316) should be cleaner.

#### 4. validate_deck — ALL commander comparisons (~20 occurrences)
**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`
**Lines:** 209, 237, 274, and many more
```rust
.find(|e| e.card.scryfall_data.id == commander_id)
```
**Fix:** Need to resolve `commander_id` (scryfall_data_id from profile) to oracle_id before validation begins. Then compare:
```rust
.find(|e| e.card.scryfall_data.oracle_id == Some(commander_oracle_id))
```
**Approach:** Add an oracle_id resolution step at the top of `validate_deck`. Accept a lookup map or resolve inline. The `DeckCommandZone` already provides the actual Card objects — we can get oracle_id from those:
```rust
let commander_oracle_id = command_zone.commander
    .and_then(|c| c.scryfall_data.oracle_id);
```
Then use `commander_oracle_id` for all comparisons instead of `profile.commander_id`.

#### 5. validate_deck tests
**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs:493-610`
```rust
let commander_id = card.scryfall_data.id;
profile.commander_id = Some(commander_id);
```
**Fix:** Keep storing scryfall_data.id (that's correct — tests simulate real behavior). The comparison logic change in validate_deck will handle the rest.

#### 6. Frontend deck view — command zone card lookup
**File:** `zwiper/src/lib/inbound/screens/deck/card/view.rs:139-177`
```rust
entries.iter().position(|e| e.card.scryfall_data.id == commander_id)
```
**Fix:** Resolve commander_id to oracle_id first (fetch the card or look up oracle_id), then compare by oracle_id. Since we fetch the card anyway (line 145), we can get oracle_id from that.

### GREEN — Correctly uses scryfall_data_id (no change needed)

| Location | Why |
|----------|-----|
| `deck_cards` table: stores both scryfall_data_id + oracle_id | Printing + identity both tracked |
| `deck_cards` INSERT/UPDATE/DELETE WHERE clauses | Row-level ops on specific printing |
| `card_profiles.scryfall_data_id` FK | 1:1 with scryfall_data |
| `HttpUpdateDeckCard.scryfall_data_id` | Changing printing selection |
| `PrintingSheet` comparisons | Comparing specific printings |
| `get_card` endpoint | Fetching a specific printing |
| `deck_card_map` join in get_deck (services.rs:170) | Joining rows to their card data |
| Card sync/upsert | Syncing printings from Scryfall |
| `DeckWarning.scryfall_data_id` | Identifying which card triggered warning |
| Frontend remove screen operations | Deleting/updating specific deck_card rows |
| Frontend add screen exclusion set (add.rs:121-130) | De-dupes search results by printing — fine since search returns one per oracle_id |
| Frontend deck_ids check (add.rs:131) | Already uses oracle_id correctly |

### YELLOW — Import band-aid to simplify after fix
**File:** `zerver/src/lib/domain/deck/services.rs:295-316`
The extra `get_multiple_scryfall_data` call to resolve oracle_ids can be simplified. Once validate_deck and other comparisons consistently resolve via oracle_id, this pattern becomes the standard approach rather than a workaround. Keep it but clean up the comment.

## Implementation Order

### Phase 1: validate_deck (zwipe-core, pure logic)
- At the top of `validate_deck`, resolve command zone card oracle_ids from `DeckCommandZone`:
  ```rust
  let commander_oid = command_zone.commander.and_then(|c| c.scryfall_data.oracle_id);
  let partner_oid = command_zone.partner_commander.and_then(|c| c.scryfall_data.oracle_id);
  let bg_oid = command_zone.background.and_then(|c| c.scryfall_data.oracle_id);
  let spell_oid = command_zone.signature_spell.and_then(|c| c.scryfall_data.oracle_id);
  ```
- Replace all `profile.commander_id` comparisons against `scryfall_data.id` with oracle_id comparisons
- Update tests

### Phase 2: Backend service layer
- `create_deck_card`: resolve commander_id → oracle_id before comparing against request
- `get_deck`: resolve commander_id → oracle_id for the entries check (fetch still uses scryfall_data_id)
- Clean up import band-aid comments

### Phase 3: Frontend
- `view.rs`: resolve command zone IDs to oracle_id before searching entries
- Create/edit screens are fine — they store scryfall_data.id, which is correct

## No Migration Needed

The database columns stay as-is. The fix is purely in comparison logic.
