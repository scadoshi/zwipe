# Clone Deck Endpoint — Backend

## Context

Users want a single-click "clone deck" action: pick one of their decks, give it a new name, and get a fresh copy with the same profile (format, commander, partner, background, signature spell) and every entry (main / sideboard / maybeboard with quantities and exact printings). Useful for brewing variants without touching the original and for versioning a proven shell.

This plan covers the **backend endpoint only**, per the user's request ("just takes deck id and clone's name then does all of the work for the user requesting"). Frontend button + client trait are a follow-up scoped in a final section.

The endpoint accepts the **source deck id** in the URL and the **new name** in the body. The server does everything else: owner lookup from JWT, authorization check on the source, deck count limit enforcement, and a single-transaction copy of the profile + all entries.

## Design decisions (defaults with override notes)

| Decision | Default | Notes |
|---|---|---|
| **Auth scope** | Caller must own source deck | Use existing `get_deck` pattern which checks `user_id != request.user_id → Forbidden`. Future public/shared-deck cloning extends here. |
| **Name collision** | Return `Duplicate` error (422) | Matches `CreateDeckProfileError::Duplicate` behavior. Client re-prompts with a different name. Alternative: auto-suffix. |
| **Deck count limit** | Enforced | Same `MAX_DECKS_PER_USER` / `UNVERIFIED_MAX_DECKS_PER_USER` as `create_deck_profile`. |
| **Commander printing** | Copied verbatim | `scryfall_data_id` for commander/partner/background/signature_spell is preserved, not refreshed to latest printing. |
| **Return shape** | Full `Deck` aggregate | Profile + entries + warnings, so frontend can navigate to the new deck without a follow-up GET. |
| **Transaction** | Single transaction | Insert new deck row + bulk-insert all entries atomically. If anything fails, nothing persists. |
| **Card copy** | All boards preserved | Main, sideboard, maybeboard all carried over with their quantities. |

## File-by-file plan

All paths are relative to the repo root.

### 1. HTTP contract

**Edit:** `zwipe-core/src/http/contracts/deck.rs` — add at the bottom:

```rust
/// POST /api/deck/{source_id}/clone body. The path param provides the
/// source deck id; the caller is taken from the JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCloneDeck {
    pub new_name: String,
}
```

(Lives alongside existing `HttpCreateDeckProfile` / `HttpUpdateDeckProfile`.)

### 2. Path constant

**Edit:** `zwipe-core/src/http/paths.rs` (around line 148, after the existing deck routes):

```rust
pub fn clone_deck_route(source_deck_id: Uuid) -> String {
    format!("/api/deck/{}/clone", source_deck_id)
}
```

### 3. Domain request type

**New file:** `zwipe-core/src/domain/deck/requests/clone_deck.rs`

```rust
use crate::domain::deck::models::deck_name::{DeckName, InvalidDeckname};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Request to clone an existing deck owned by `user_id` into a new deck
/// with `new_name`, also owned by `user_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneDeck {
    pub source_deck_id: Uuid,
    pub new_name: DeckName,
    pub user_id: Uuid,
    pub email_verified: bool,
}

impl CloneDeck {
    pub fn new(
        source_deck_id: Uuid,
        new_name: String,
        user_id: Uuid,
        email_verified: bool,
    ) -> Result<Self, InvalidCloneDeck> {
        let new_name = DeckName::new(new_name)?;
        Ok(Self {
            source_deck_id,
            new_name,
            user_id,
            email_verified,
        })
    }
}

#[derive(Debug, Error)]
pub enum InvalidCloneDeck {
    #[error(transparent)]
    Name(#[from] InvalidDeckname),
}
```

**Edit:** `zwipe-core/src/domain/deck/requests/mod.rs` — add the module and re-exports:

```rust
pub mod clone_deck;
pub use clone_deck::{CloneDeck, InvalidCloneDeck};
```

### 4. Service-layer error type

**New file:** `zerver/src/lib/domain/deck/models/deck/clone_deck.rs`

```rust
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CloneDeckError {
    #[error("source deck not found")]
    SourceNotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("a deck with that name already exists")]
    Duplicate,
    #[error("deck count limit reached")]
    LimitReached,
    #[error(transparent)]
    GetSource(#[from] GetDeckProfileError),
    #[error(transparent)]
    DeckFromDb(anyhow::Error),
    #[error(transparent)]
    Database(anyhow::Error),
}
```

**Edit:** `zerver/src/lib/domain/deck/models/deck/mod.rs` — add `pub mod clone_deck;` and `pub use clone_deck::CloneDeckError;`.

### 5. Repository port method

**Edit:** `zerver/src/lib/domain/deck/ports.rs` — add to the `DeckRepository` trait (alongside the existing create/update/delete methods, around line 75):

```rust
fn clone_deck(
    &self,
    source: &Deck,
    new_name: &DeckName,
    owner_id: Uuid,
) -> impl Future<Output = Result<Deck, CloneDeckError>> + Send;
```

And add to the `DeckService` trait:

```rust
fn clone_deck(
    &self,
    request: &CloneDeck,
) -> impl Future<Output = Result<Deck, CloneDeckError>> + Send;
```

(Plus a `use crate::domain::deck::models::deck::clone_deck::CloneDeckError;` at the top of the file, and `use zwipe_core::domain::deck::requests::CloneDeck;`.)

### 6. Repository SQLx implementation

**Edit:** `zerver/src/lib/outbound/sqlx/deck/mod.rs` — add a new method inside the `impl DeckRepository for Sqlx` block. The method takes the already-fetched `source: &Deck` (fetched by the service layer via `get_deck`, which handles the authorization check) and atomically inserts the new profile + all entries.

```rust
async fn clone_deck(
    &self,
    source: &Deck,
    new_name: &DeckName,
    owner_id: Uuid,
) -> Result<Deck, CloneDeckError> {
    let mut tx = self.pool.begin().await
        .map_err(|e| CloneDeckError::Database(e.into()))?;

    // 1. Insert new deck profile row, returning the full DatabaseDeckProfile.
    //    Copies format, commander_id, partner_commander_id, background_id,
    //    signature_spell_id from source. Owner is the caller. Name is
    //    supplied. Uses the same RETURNING shape as create_deck_profile so
    //    the returned row already has card_count=0 and the cached name
    //    fields resolved.
    let new_profile = sqlx::query_as!(
        DatabaseDeckProfile,
        r#"
        WITH new_deck AS (
            INSERT INTO decks (
                name, commander_id, partner_commander_id, background_id,
                signature_spell_id, format, user_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
        )
        SELECT
            nd.id,
            nd.name,
            nd.commander_id,
            nd.partner_commander_id,
            nd.background_id,
            nd.signature_spell_id,
            nd.format,
            nd.user_id,
            0::BIGINT AS "card_count!",
            (SELECT name FROM scryfall_data WHERE id = nd.commander_id) AS commander_name,
            (SELECT name FROM scryfall_data WHERE id = nd.partner_commander_id) AS partner_commander_name,
            (SELECT name FROM scryfall_data WHERE id = nd.background_id) AS background_name,
            (SELECT name FROM scryfall_data WHERE id = nd.signature_spell_id) AS signature_spell_name
        FROM new_deck nd
        "#,
        new_name.to_string(),
        source.deck_profile.commander_id,
        source.deck_profile.partner_commander_id,
        source.deck_profile.background_id,
        source.deck_profile.signature_spell_id,
        source.deck_profile.format.as_ref().map(|f| f.to_string()),
        owner_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("unique_deck_name_per_user") => {
            CloneDeckError::Duplicate
        }
        _ => CloneDeckError::Database(e.into()),
    })?;

    let new_deck_id = new_profile.id;

    // 2. Bulk insert all entries from source under the new deck id.
    //    Use a single multi-row INSERT via QueryBuilder. Preserves board,
    //    quantity, oracle_id, and scryfall_data_id verbatim.
    if !source.entries.is_empty() {
        let mut builder = sqlx::QueryBuilder::new(
            "INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, board) "
        );
        builder.push_values(&source.entries, |mut b, entry| {
            b.push_bind(new_deck_id)
                .push_bind(entry.deck_card.scryfall_data_id)
                .push_bind(entry.deck_card.oracle_id)
                .push_bind(entry.deck_card.quantity.value() as i32)
                .push_bind(entry.deck_card.board.as_str());
        });
        builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(|e| CloneDeckError::Database(e.into()))?;
    }

    tx.commit().await
        .map_err(|e| CloneDeckError::Database(e.into()))?;

    // 3. Convert profile + rehydrate full Deck aggregate. Entries we already
    //    have in memory — just rebuild DeckEntry with the new deck_id on
    //    each DeckCard. Warnings are recomputed by validate_deck.
    let new_profile: DeckProfile = new_profile
        .try_into()
        .map_err(|e: IntoDeckProfileError| CloneDeckError::DeckFromDb(e.into()))?;

    let new_entries: Vec<DeckEntry> = source
        .entries
        .iter()
        .map(|e| DeckEntry {
            card: e.card.clone(),
            deck_card: DeckCard {
                deck_id: new_deck_id,
                scryfall_data_id: e.deck_card.scryfall_data_id,
                oracle_id: e.deck_card.oracle_id,
                quantity: e.deck_card.quantity,
                board: e.deck_card.board.clone(),
            },
        })
        .collect();

    let warnings = validate_deck(&new_profile, &new_entries);
    Ok(Deck {
        deck_profile: new_profile,
        entries: new_entries,
        warnings,
    })
}
```

Note the SQLx prepare cache: after adding the `query_as!` macro, run `cargo sqlx prepare --workspace` from repo root before committing (per `context/CLAUDE.md`).

### 7. Service implementation

**Edit:** `zerver/src/lib/domain/deck/services.rs` — add inside `impl<DR, CR> DeckService for Service<DR, CR>`:

```rust
async fn clone_deck(&self, request: &CloneDeck) -> Result<Deck, CloneDeckError> {
    // 1. Load the source deck. `get_deck` already enforces:
    //    - NotFound if the id doesn't exist
    //    - Forbidden if user_id doesn't match the deck's user_id
    let source = self
        .get_deck(&GetDeckProfile::new(request.user_id, request.source_deck_id))
        .await
        .map_err(|e| match e {
            GetDeckError::NotFound => CloneDeckError::SourceNotFound,
            GetDeckError::Forbidden => CloneDeckError::Forbidden,
            other => CloneDeckError::GetSource(other.into()),
        })?;

    // 2. Deck count limit (same rule as create_deck_profile).
    let deck_count = self.deck_repo
        .count_decks_by_user(request.user_id)
        .await
        .map_err(CloneDeckError::Database)?;
    let limit = if request.email_verified {
        MAX_DECKS_PER_USER
    } else {
        UNVERIFIED_MAX_DECKS_PER_USER
    };
    if deck_count >= limit as i64 {
        return Err(CloneDeckError::LimitReached);
    }

    // 3. Delegate to repo for the transactional insert.
    self.deck_repo
        .clone_deck(&source, &request.new_name, request.user_id)
        .await
}
```

(Adjust the exact error-conversion glue to match the real `GetDeckError` shape from `services.rs`. The point is: source-not-found and forbidden get mapped to dedicated `CloneDeckError` variants.)

### 8. HTTP handler

**New file:** `zerver/src/lib/inbound/http/handlers/deck/clone_deck.rs`

```rust
use crate::domain::deck::models::deck::clone_deck::CloneDeckError;
use crate::inbound::http::errors::ApiError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::state::AppState;
use crate::domain::deck::ports::DeckService;
use crate::domain::user::ports::UserService;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use uuid::Uuid;
use zwipe_core::domain::deck::models::deck::Deck;
use zwipe_core::domain::deck::requests::CloneDeck;
use zwipe_core::domain::user::requests::GetUser;
use zwipe_core::http::contracts::deck::HttpCloneDeck;

impl From<CloneDeckError> for ApiError {
    fn from(value: CloneDeckError) -> Self {
        match value {
            CloneDeckError::SourceNotFound => Self::NotFound("source deck not found".into()),
            CloneDeckError::Forbidden => Self::Forbidden("cannot clone another user's deck".into()),
            CloneDeckError::Duplicate => {
                Self::UnprocessableEntity("a deck with that name already exists".into())
            }
            CloneDeckError::LimitReached => {
                Self::UnprocessableEntity("deck count limit reached".into())
            }
            CloneDeckError::GetSource(e) => e.into(),
            CloneDeckError::DeckFromDb(e) => e.log_500(),
            CloneDeckError::Database(e) => e.log_500(),
        }
    }
}

pub async fn clone_deck<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(source_deck_id): Path<Uuid>,
    Json(body): Json<HttpCloneDeck>,
) -> Result<(StatusCode, Json<Deck>), ApiError>
where
    US: UserService,
    DS: DeckService,
{
    // Pull email_verified the same way create_deck_profile does.
    let db_user = state
        .user_service
        .get_user(&GetUser::from(user.id))
        .await?;
    let email_verified = db_user.email_verified_at.is_some();

    // Build + validate the domain request.
    let request = CloneDeck::new(source_deck_id, body.new_name, user.id, email_verified)
        .map_err(|e| ApiError::UnprocessableEntity(e.to_string()))?;

    let new_deck = state
        .deck_service
        .clone_deck(&request)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(new_deck)))
}
```

**Edit:** `zerver/src/lib/inbound/http/handlers/deck/mod.rs` — `pub mod clone_deck;` and `pub use clone_deck::clone_deck;`.

### 9. Route registration

**Edit:** `zerver/src/lib/inbound/http/routes.rs` (around line 220, inside the `/api/deck` nest):

```rust
.route("/{deck_id}/clone", post(clone_deck))
```

Place it adjacent to `get_deck` / `update_deck_profile` / `delete_deck` so all `/{deck_id}/*` routes are grouped.

Add `clone_deck` to the imports at the top of the file with the other deck handlers.

### 10. SQLx prepare cache

After all code changes land and compile:

```bash
cargo sqlx prepare --workspace
```

Commit the generated `.sqlx/` updates. CI builds use offline mode and will fail without this.

## Files changed

**New files:**
- `zwipe-core/src/domain/deck/requests/clone_deck.rs`
- `zerver/src/lib/domain/deck/models/deck/clone_deck.rs`
- `zerver/src/lib/inbound/http/handlers/deck/clone_deck.rs`

**Edited files:**
- `zwipe-core/src/http/contracts/deck.rs` — add `HttpCloneDeck`
- `zwipe-core/src/http/paths.rs` — add `clone_deck_route`
- `zwipe-core/src/domain/deck/requests/mod.rs` — re-export `CloneDeck`
- `zerver/src/lib/domain/deck/models/deck/mod.rs` — re-export `CloneDeckError`
- `zerver/src/lib/domain/deck/ports.rs` — add `clone_deck` to `DeckRepository` and `DeckService` traits
- `zerver/src/lib/outbound/sqlx/deck/mod.rs` — implement `clone_deck` on the repo
- `zerver/src/lib/domain/deck/services.rs` — implement `clone_deck` on the service
- `zerver/src/lib/inbound/http/handlers/deck/mod.rs` — re-export the handler
- `zerver/src/lib/inbound/http/routes.rs` — register `POST /api/deck/{deck_id}/clone`
- `.sqlx/` — regenerated offline cache

## Reused infrastructure

- `CreateDeckProfile`-style domain request + validation pattern (`DeckName::new`)
- `DatabaseDeckProfile` SQLx row struct and its `TryFrom` conversion to domain `DeckProfile`
- `QueryBuilder::push_values` for bulk entry insert (mirrors how `bulk_create_deck_cards` does import)
- Existing `validate_deck(&profile, &entries)` for the warnings field on the returned `Deck`
- `GetDeckProfile::new(user_id, deck_id)` + `DeckService::get_deck` — does the owner auth check for free
- `AuthenticatedUser` extractor, `ApiError::Forbidden` / `NotFound` / `UnprocessableEntity` variants
- `MAX_DECKS_PER_USER` / `UNVERIFIED_MAX_DECKS_PER_USER` constants from `services.rs`
- `unique_deck_name_per_user` DB constraint for duplicate detection
- SQLx transaction pattern from `create_deck_profile` (begin → query_as → commit)

## Verification

1. **Compile and lint:**
   ```bash
   cargo check --workspace
   cargo sqlx prepare --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

2. **Manual end-to-end via curl** (after `cargo run --bin zerver`):
   ```bash
   TOKEN="<jwt from login response>"
   SOURCE_ID="<uuid of an existing deck owned by caller>"

   # Happy path
   curl -X POST http://localhost:8080/api/deck/$SOURCE_ID/clone \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"new_name": "My Clone"}' | jq

   # Expect 201 with full Deck JSON. Verify:
   # - deck_profile.id differs from $SOURCE_ID
   # - deck_profile.user_id matches caller
   # - deck_profile.name == "My Clone"
   # - entries.len() matches source
   # - every entry has the new deck_id and preserved board/quantity/oracle_id
   # - commander_id / partner / background / signature_spell preserved
   ```

3. **Error cases:**
   - Clone with a name that already exists → 422 Duplicate
   - Clone a deck you don't own → 403 Forbidden
   - Clone a non-existent deck id → 404 NotFound
   - Clone while at deck-count limit → 422 LimitReached
   - Clone with an empty or too-long name → 422 (validated by `DeckName::new`)
   - Clone with invalid JWT → 401 Unauthorized (existing middleware)

4. **DB invariants after clone:**
   - New row in `decks` with `user_id = caller`
   - Same count of rows in `deck_cards` as source, all with `deck_id = new_deck_id`
   - `UNIQUE(deck_id, oracle_id)` holds (trivially, since `deck_id` is fresh)
   - Source deck is completely untouched

5. **Transactional atomicity check:**
   - Temporarily force a failure mid-insert (e.g. manually corrupt one entry's `scryfall_data_id` in a dev database to an invalid uuid). Verify no new deck row is left behind after the request fails — the transaction rolls back cleanly.

## Frontend follow-up (out of scope for this plan)

Once the endpoint is live, the frontend work is:

- `zwiper/src/lib/outbound/client/deck/clone_deck.rs` — new `ClientCloneDeck` trait implementing `POST` against `clone_deck_route(source_id)` with the Bearer token and `HttpCloneDeck` body. Mirror the existing `ClientCreateDeck` pattern.
- Deck view screen (`zwiper/src/lib/inbound/screens/deck/view.rs` or similar) — add a "clone" button to the util-bar next to back/filter/refresh. On click, prompt for a new name (default suggestion: `"{source.name} (copy)"`), call `client.clone_deck()`, toast the result, and navigate to the new deck's view screen.
- Session upkeep + error handling follows the existing `ClientCreateDeck` pattern.
- Optional: add the same button to the deck list screen for a faster "clone from list" flow.
