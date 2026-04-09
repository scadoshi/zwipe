# Clone Deck — As-Built Reference

> Companion to `clone-deck.md` (pre-implementation plan). This document
> describes what actually shipped, where to look if it breaks, and how to
> roll back or rewrite cleanly. Read this first if the feature is
> misbehaving in production.

## What it does

A logged-in user opens any deck they own, taps **more → clone deck**, and a modal prompts them for a new deck name (prefilled with `"{source name} (clone)"`). On save, the backend transactionally copies the deck profile and every `deck_cards` row under a new deck id owned by the caller, and the frontend navigates to the new deck's view screen.

## Git history

Two commits, in order. The frontend commit depends on the backend commit being deployed first.

| Commit | Scope | Files |
|---|---|---|
| `ff367d86` | Backend endpoint | 14 files (zerver + zwipe-core + `.sqlx/`) |
| `fdf01a99` | Frontend dialog + client trait | 6 files (zwiper only) |

**Rollback path:** `git revert fdf01a99` first (frontend only, safe anytime), then `git revert ff367d86` (backend, only after frontend revert is deployed).

## Endpoint contract

```
POST /api/deck/{source_deck_id}/clone
Authorization: Bearer <jwt>
Content-Type: application/json

Request body:
{"new_name": "My Clone"}

Success (201 Created):
{"deck_id": "<new-uuid>"}

Error responses:
404 — source deck not found
403 — caller does not own source deck
422 — duplicate name | deck count limit reached | invalid name
401 — missing / invalid JWT
```

The response is deliberately minimal — **just the new deck id**. The frontend navigates to `Router::ViewDeck { deck_id }` which loads the full aggregate via its own resources. Do not change this contract to return the full `Deck` — it's intentionally lightweight.

## End-to-end flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. USER CLICKS "clone deck" in MoreButtons bottom sheet         │
│    more_buttons.rs: show_more_sheet.set(false);                 │
│                     show_clone_dialog.set(true)                 │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│ 2. CloneDeckDialog mounts/renders in view.rs                    │
│    use_effect resets `new_name` to "{source} (clone)" prefill   │
│    Save button disabled while empty or is_cloning               │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│ 3. USER TAPS SAVE                                               │
│    session.upkeep(client) → refresh token if needed             │
│    client().clone_deck(source_id, &HttpCloneDeck, &session)     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│ 4. HTTP POST to /api/deck/{source_id}/clone                     │
│    Handler extracts AuthenticatedUser + Path(source_id) + body  │
│    Fetches email_verified via user_service.get_user()           │
│    Builds CloneDeck domain request (validates DeckName)         │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│ 5. DeckService::clone_deck                                      │
│    a) deck_repo.get_deck_profile(source) → NotFound | Forbidden │
│    b) deck_repo.count_decks_by_user(user) → limit check         │
│    c) deck_repo.clone_deck(source, new_name, owner)             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│ 6. Postgres repository (single transaction)                     │
│    BEGIN                                                        │
│    INSERT INTO decks (name,commander_id,...,format,user_id)     │
│      SELECT $1, commander_id, ..., format, $2                   │
│      FROM decks WHERE id = $3 RETURNING id;                     │
│    INSERT INTO deck_cards (deck_id,scryfall_data_id,...,board)  │
│      SELECT $1, scryfall_data_id, ..., board                    │
│      FROM deck_cards WHERE deck_id = $2;                        │
│    COMMIT                                                       │
│    → returns new deck_id (Uuid)                                 │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│ 7. Response: 201 Created, {"deck_id": "<uuid>"}                 │
│    Frontend: toast.info("cloned as ..."), close modal,          │
│    navigator.push(Router::ViewDeck { deck_id: cloned.deck_id }) │
│    New view screen loads profile/entries via existing resources │
└─────────────────────────────────────────────────────────────────┘
```

## File map

### Backend (`ff367d86`)

| Path | Role |
|---|---|
| `zwipe-core/src/http/contracts/deck.rs` | `HttpCloneDeck` request + `HttpClonedDeck` response structs |
| `zwipe-core/src/http/paths.rs` | `clone_deck_route(source_deck_id)` path helper (auto re-exported through `zerver/src/lib/inbound/http/routes.rs`'s `pub use zwipe_core::http::paths::*`) |
| `zwipe-core/src/domain/deck/requests/clone_deck.rs` | `CloneDeck` domain request + `InvalidCloneDeck` validation error. Wraps `DeckName` so profanity / length validation happens at construction. |
| `zerver/src/lib/domain/deck/models/deck/clone_deck.rs` | `CloneDeckError` service-layer error (has `From<sqlx::Error>` impl that maps unique-constraint violations to `Duplicate` via `IsConstraintViolation` trait) |
| `zerver/src/lib/domain/deck/ports.rs` | `clone_deck` method on both `DeckRepository` and `DeckService` traits |
| `zerver/src/lib/outbound/sqlx/deck/mod.rs` | `async fn clone_deck` — two `INSERT ... SELECT` statements in a transaction. Takes `source_deck_id`, `&DeckName`, `owner_id`; returns `Uuid`. No Rust-side iteration over entries. |
| `zerver/src/lib/domain/deck/services.rs` | `async fn clone_deck` — ownership check via `get_deck_profile`, count-limit check, then delegates to repo |
| `zerver/src/lib/inbound/http/handlers/deck/clone_deck.rs` | HTTP handler + `From<CloneDeckError> for ApiError` + `From<InvalidCloneDeck> for ApiError` |
| `zerver/src/lib/inbound/http/routes.rs` | Route registration: `.route("/{deck_id}/clone", post(clone_deck))` inside the `/api/deck` nest |
| `.sqlx/query-039c...json`, `.sqlx/query-063a...json` | SQLx offline cache entries for the two new queries — **required for CI** |

### Frontend (`fdf01a99`)

| Path | Role |
|---|---|
| `zwiper/src/lib/outbound/client/deck/clone_deck.rs` | `ClientCloneDeck` trait + impl on `ZwipeClient`. POSTs `HttpCloneDeck` body, parses `HttpClonedDeck` response, returns `Result<HttpClonedDeck, ApiError>` |
| `zwiper/src/lib/outbound/client/deck/mod.rs` | Registers `clone_deck` module |
| `zwiper/src/lib/inbound/screens/deck/components/clone_deck_dialog.rs` | Self-contained dialog component. Props: `source_deck_id: Uuid`, `default_name: String`, `open: Signal<bool>`. Manages `new_name` and `is_cloning` signals locally. |
| `zwiper/src/lib/inbound/screens/deck/components/mod.rs` | Registers `clone_deck_dialog` module |
| `zwiper/src/lib/inbound/screens/deck/components/more_buttons.rs` | New `show_clone_dialog: Signal<bool>` prop. Button sits between "export cards" and "delete deck". |
| `zwiper/src/lib/inbound/screens/deck/view.rs` | New `show_clone_dialog` signal, prop passthrough, dialog mount with `default_name` computed from `deck_profile_resource` |

## Debugging playbook

**Symptom: Button does nothing when tapped**
1. Check `more_buttons.rs` — is `show_clone_dialog.set(true)` actually on the onclick?
2. Check `view.rs` — is `show_clone_dialog` being passed to `MoreButtons` as a prop?
3. Check `CloneDeckDialog` is mounted inside the `Bouncer` block — if it's outside the screen root, `AlertDialogRoot` won't show.

**Symptom: Modal opens but prefill is empty**
1. `deck_profile_resource()` hasn't resolved yet when the user opens the dialog. The `unwrap_or_default()` fallback is `""`.
2. Not a bug per se — placeholder `"new deck name"` appears. Save button stays disabled until the user types.
3. Fix (if desired): disable the "more" button until `deck_profile_resource().is_some_and(|r| r.is_ok())`, or pre-compute the prefill in a `Signal<String>` that updates via `use_effect`.

**Symptom: 422 "a deck with that name already exists"**
- Expected behavior when the caller already owns a deck with the same name. The modal stays open so the user can rename.
- Constraint enforced by Postgres: `CONSTRAINT unique_deck_name_per_user UNIQUE(user_id, name)` in migration `20250810194454_create_decks.sql`.
- Translation lives in `outbound/sqlx/deck/error.rs` via `IsConstraintViolation::is_unique_constraint_violation` — see `From<sqlx::Error> for CloneDeckError` in `domain/deck/models/deck/clone_deck.rs`.

**Symptom: 422 "deck limit reached"**
- `MAX_DECKS_PER_USER = 20` (verified) / `UNVERIFIED_MAX_DECKS_PER_USER = 1` (unverified email) in `zerver/src/lib/domain/deck/mod.rs`.
- Check `email_verified_at` column on the `users` row.
- The limit check is in `DeckService::clone_deck`, mirroring `create_deck_profile`.

**Symptom: 403 "cannot clone another user's deck"**
- Reuses `get_deck_profile`'s ownership check (returns `GetDeckProfileError::Forbidden` when `user_id` on deck doesn't match caller). Mapped to `CloneDeckError::Forbidden` in the service.
- This should be unreachable from the current UI since the clone button only appears on decks the caller owns. If it fires, suspect session bleed or stale JWT.

**Symptom: 500 Internal Server Error (DB)**
- Check server logs via `LOG_DIR` env var.
- Most likely cause: `.sqlx/` offline cache out of sync with schema. Run `cargo sqlx prepare --workspace` and commit.
- Second most likely: the source deck was deleted between the `get_deck_profile` check and the repo call (race). Falls through to a database error on the `INSERT ... SELECT FROM ... WHERE id = $3` which will find zero rows and return `sqlx::Error::RowNotFound`. This becomes `CloneDeckError::Database` (generic 500) — it _should_ become a 404 but currently doesn't. Low priority.

**Symptom: Navigates to new deck view but it's blank**
- The navigate happens before the new deck's view resources load. Normal — should show the spinner briefly, then populate.
- If it stays blank: `get_deck` on the new id is failing. Check that the backend transaction committed (`tx.commit()` at end of repo method) and that `deck_cards` rows were actually inserted.

**Symptom: Clone "works" but only copies profile, not cards**
- The second `INSERT INTO deck_cards ... SELECT` statement didn't run or returned zero rows. Should be impossible unless source deck has zero cards (empty deck) — in which case it's correct behavior.
- To verify: `SELECT count(*) FROM deck_cards WHERE deck_id = '<source>'` and compare to `<new>`.

**Symptom: Clone "works" but format/commander/etc are wrong**
- Those fields come from the SQL `SELECT commander_id, ..., format FROM decks WHERE id = $3` column-to-column. If they differ, the bug is not in clone — the source deck already had the wrong values.

## Reverting / rewriting

If the feature needs to be pulled:

```bash
git revert fdf01a99                     # frontend first (user-visible)
git revert ff367d86                     # backend second (after frontend ships)
cargo sqlx prepare --workspace          # regenerate cache (will drop clone entries)
git add .sqlx && git commit --amend --no-edit
```

If a rewrite is needed, the **most important invariant** to preserve is:
- The repo uses pure `INSERT ... SELECT` with no Rust-side hydration of entries. This is O(1) on the Rust side regardless of deck size. Do not regress to a `QueryBuilder::push_values` loop that loads entries into memory.
- The response is `HttpClonedDeck { deck_id }`, not the full `Deck`. The frontend's `ViewDeck` screen loads everything itself via existing resources. Returning the full aggregate would be wasted work and serialization.
- The dialog is a separate component file (mirrors `DeleteAccountDialog` in `profile/components/`). Don't inline its state into `view.rs` — `view.rs` is already too large.

## Testing checklist

See `clone-deck.md` → "Verification" section for the full backend e2e. Frontend-only sanity:

- [ ] Button visible in "more actions" sheet, second-to-last position
- [ ] Tapping button closes the sheet and opens the modal
- [ ] Input prefilled with `"{source name} (clone)"` on first open
- [ ] Save button disabled with empty input
- [ ] Save navigates to new deck view + shows info toast
- [ ] New deck has same profile (commander, format, etc.) and entries as source
- [ ] Source deck is unchanged (GET and compare)
- [ ] Cancel closes modal without API call
- [ ] Reopening after cancel re-seeds the prefill (doesn't keep the typed value)
- [ ] Duplicate name shows error toast, modal stays open, typed name preserved
- [ ] Name >64 chars shows error toast (caught at `DeckName::new` on the backend)
