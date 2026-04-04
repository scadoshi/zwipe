# Card Domain Proxy Cleanup

## Context

Continuation of zerver pub import cleanup. User, auth, deck, logo, moderation, paths, helpers are done. Card is the last domain — largest scope with 17 pure proxy files, 5 mixed mod.rs files, ~11 zerver import rewrites, and ~34 zwiper import rewrites.

## Step 1: Delete pure proxy files (17 files)

```
zerver/src/lib/domain/card/models/card_profile/mod.rs  + rmdir card_profile/
zerver/src/lib/domain/card/models/scryfall_data/all_parts.rs
zerver/src/lib/domain/card/models/scryfall_data/card_faces.rs
zerver/src/lib/domain/card/models/scryfall_data/colors.rs
zerver/src/lib/domain/card/models/scryfall_data/image_uris.rs
zerver/src/lib/domain/card/models/scryfall_data/legalities.rs
zerver/src/lib/domain/card/models/scryfall_data/prices.rs
zerver/src/lib/domain/card/models/scryfall_data/rarity.rs
zerver/src/lib/domain/card/models/search_card/card_type.rs
zerver/src/lib/domain/card/models/search_card/filter_cards.rs
zerver/src/lib/domain/card/models/search_card/group_cards.rs
zerver/src/lib/domain/card/models/search_card/stop_words.rs
zerver/src/lib/domain/card/models/search_card/card_filter/error.rs
zerver/src/lib/domain/card/models/search_card/card_filter/order_by_option.rs
zerver/src/lib/domain/card/models/search_card/card_filter/getters.rs  (comment only)
zerver/src/lib/domain/card/models/search_card/card_filter/builder/getters.rs  (comment only)
zerver/src/lib/domain/card/models/search_card/card_filter/builder/setters.rs  (comment only)
```

## Step 2: Cascade-delete empty directories

After deleting children:
- `scryfall_data/mod.rs` → remove all 7 `pub mod` lines + `pub use ScryfallData` → becomes empty → delete + rmdir
- `card_filter/builder/mod.rs` → remove `pub use` + all `pub mod` lines → becomes empty → delete + rmdir
- `card_filter/mod.rs` → remove `pub use` + `pub mod builder/error/getters/order_by_option` → becomes empty → delete + rmdir
- `search_card/mod.rs` → remove `pub mod card_type/filter_cards/group_cards/stop_words/card_filter` → only `pub mod error;` remains (server type) → KEEP
- `card_profile/` dir → already empty after deleting mod.rs

## Step 3: Edit remaining parent/mixed files

- `card/models/mod.rs` → remove `pub mod card_profile;`, `pub mod scryfall_data;`, `pub use zwipe_core::domain::card::Card;`. Keep `pub mod helpers;`, `pub mod search_card;`, `#[cfg] pub mod sync_metrics;`
- `search_card/mod.rs` → remove deleted submodule declarations, keep `pub mod error;`

## Step 4: Rewrite zerver imports (~11 files)

### Path mapping (old → new):
```
crate::domain::card::models::Card                        → zwipe_core::domain::card::Card
crate::domain::card::models::card_profile::CardProfile   → zwipe_core::domain::card::card_profile::CardProfile
crate::domain::card::models::scryfall_data::ScryfallData → zwipe_core::domain::card::scryfall_data::ScryfallData
crate::domain::card::models::scryfall_data::*::*         → zwipe_core::domain::card::scryfall_data::*::*
crate::domain::card::models::search_card::*              → zwipe_core::domain::card::search_card::*
```

### Types that STAY in crate::
- `SearchCardsError` (in `search_card/error.rs`)
- `SleeveScryfallData`, `SleeveCardProfile` (in `helpers.rs` — server-only traits)
- `SyncMetrics`, `SyncResult` (in `sync_metrics.rs` — server-only)
- All request error types in `card/requests/`

### Zerver files to rewrite:
- `domain/card/models/helpers.rs` — CardProfile, ScryfallData, Card
- `domain/card/requests/get_card_profile.rs` — ScryfallData
- `domain/card/requests/get_scryfall_data.rs` — check for card model imports
- `outbound/sqlx/card/models.rs` — 8 scryfall subtype imports (AllParts, CardFaces, Colors, etc.)
- `outbound/sqlx/card/mod.rs` — Card, CardFilter, OrderByOption, PLAYABLE_LAYOUTS, CardProfile, ScryfallData
- `outbound/sqlx/card/card_profile.rs` — CardProfile
- `outbound/sqlx/card/helpers/scryfall_data_fields.rs` — ScryfallData
- `outbound/sqlx/card/helpers/upsert_card.rs` — CardProfile, ScryfallData, SleeveScryfallData (stays)
- `outbound/sqlx/card/sync_metrics.rs` — check imports
- `inbound/external/scryfall/planeswalker.rs` — ScryfallData
- `domain/card/models/search_card/error.rs` — check (uses card request types, likely stays)

## Step 5: Rewrite zwiper imports (~34 files)

### Path mapping:
```
zwipe::domain::card::models::Card                                    → zwipe_core::domain::card::Card
zwipe::domain::card::models::scryfall_data::*                        → zwipe_core::domain::card::scryfall_data::*
zwipe::domain::card::models::search_card::card_filter::*             → zwipe_core::domain::card::search_card::card_filter::*
zwipe::domain::card::models::search_card::card_filter::builder::*    → zwipe_core::domain::card::search_card::card_filter::builder::*
zwipe::domain::card::models::search_card::filter_cards::*            → zwipe_core::domain::card::search_card::filter_cards::*
zwipe::domain::card::models::search_card::group_cards::*             → zwipe_core::domain::card::search_card::group_cards::*
zwipe::domain::card::models::search_card::stop_words::*              → zwipe_core::domain::card::search_card::stop_words::*
zwipe::domain::card::models::search_card::card_type::*               → zwipe_core::domain::card::search_card::card_type::*
```

### Zwiper files (34):
Split across 3 agent groups by directory:
- **Group A** (~12): `zwiper/src/lib/inbound/screens/deck/card/filter/**`
- **Group B** (~12): `zwiper/src/lib/inbound/screens/deck/{card/*, components/*, other screens}`
- **Group C** (~10): `zwiper/src/lib/inbound/{components/*, screens/home.rs}` + any remaining

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings
```
