# Deck folders — organize the deck list

**Status: PLANNED (owner 2026-07-11). Not started. Shape agreed: custom folders,
one folder per deck, collapsible grouped deck list. This doc is the build spec.**

## One sentence

Let players create their own named folders and file each deck into one, so the deck
list groups into collapsible folder sections (plus an "Uncategorized" group).

## Agreed shape (with rationale)

- **Custom, user-named folders** — not a fixed predefined set. Users expect to name
  their own (like playlists). Validation is cheap (see below), so there's no reason to
  restrict them.
- **One folder per deck** (v1) — a folder is a deck's "home." Cross-cutting labels are
  already covered by **deck tags** (`deck_tag` / `deck_other_tag`), so making folders
  many-to-many would just duplicate tags. One-per-deck keeps a clean file-folder model.
  (If a real need for a deck-in-multiple-folders appears later, revisit — but tags likely
  already serve it.)
- **Validation is NOT the hard part.** `DeckName::new`
  (`zwipe-core/.../deck/models/deck_name.rs`) already does ≤64 chars + bad-word filter
  (`ContainsBadWord`) + trim + unique-per-user. `FolderName` is a copy of that pattern.
- **The real lift is the client UI** (grouped/collapsible list + folder management), not
  the backend, which is mechanical (mirror the existing deck CRUD).

## Data model

- New table `folders`: `id UUID PK`, `user_id UUID NOT NULL REFERENCES users ON DELETE
  CASCADE`, `name VARCHAR NOT NULL`, `position INT` (manual ordering), `created_at`,
  `updated_at`, `UNIQUE(user_id, name)` (mirror `decks`' `unique_deck_name_per_user`).
- Add `decks.folder_id UUID NULL REFERENCES folders(id) ON DELETE SET NULL` — **deleting
  a folder orphans its decks to Uncategorized, never deletes them.** `NULL` = Uncategorized.
- Index `folders(user_id)`; `decks.folder_id` is already covered by `idx_decks_user_id`
  patterns (add `idx_decks_folder_id` if grouping queries need it).

## Backend (~half a day, follows existing deck patterns)

- **Domain** (`zwipe-core/src/domain/deck/`): `Folder` model + `FolderName` newtype
  (clone `DeckName`); requests `create_folder`, `rename_folder`, `delete_folder`, and a
  `folder_id` on the deck-profile update (or a dedicated `move_deck_to_folder`).
- **Repo** (`zerver/.../outbound/sqlx/deck/`): folder CRUD; include `folder_id` in the
  deck-profile row and a `get_folders(user_id)` query. `cargo sqlx prepare --workspace`.
- **HTTP** (`zerver/.../inbound/http/handlers/deck/` + `routes.rs`): `POST/GET /api/folder`,
  `PUT/DELETE /api/folder/{id}`, and thread `folder_id` through `update_deck_profile`
  (additive `Opdate`/`Option` field — see `api_evolution.md`). `Http*` contracts in
  `zwipe-core/src/http/contracts/`.
- **Tests** (`zerver/tests/`): the new integration harness makes this quick — folder CRUD,
  assign/move a deck, delete-folder-orphans-decks (assert decks survive with `folder_id`
  NULL), cross-user isolation (404), duplicate-name reject.

## Client (~1 day — the actual lift)

- **Grouped deck list** (`zwiper/.../screens/deck/list.rs`, currently a flat sorted
  `Vec<DeckProfile>`): fetch folders + decks, render **collapsible folder sections** with
  an **Uncategorized** group. Reuse the existing per-deck card render; add section headers
  + expand/collapse state (persist collapsed state in memory like the filter store).
- **Folder management sheet**: create / rename / delete a folder — same modal/bottom-sheet
  patterns as the deck forms. Delete confirms ("decks move to Uncategorized").
- **Move-to-folder**: a simple **select/picker in edit-deck** (or a per-deck "Move to…"
  action). **Skip drag-and-drop for v1** — much harder in the webview for little gain.
- Empty states: no folders yet (just the flat list), empty folder.

## Effort

**~1–1.5 focused days for a solid v1.** Backend is mechanical; client UI is the bulk.
Nothing exotic — the deck feature's structure means mostly copying established patterns.

## Open decisions (confirm before building)

1. **Manual folder ordering** (`position`) vs alphabetical — `position` is more work
   (reorder UI) but users usually want it. Could ship alphabetical v1, add reorder later.
2. **Where "move to folder" lives** — inline per-deck action on the list vs a field in
   edit-deck. (Edit-deck is simplest; a list action is more discoverable.)
3. **Collapsed-state persistence** — in-memory (forgets on restart) vs a user pref.
   In-memory is the cheap v1 (mirror the filter-persistence approach).
4. **Uncategorized placement** — top or bottom of the list.

## Sequencing

1. Backend: migration → `Folder`/`FolderName` → repo → handlers → `.sqlx` → tests.
2. Client: grouped/collapsible list → folder-management sheet → move-to-folder picker.
3. Ship WITH tests (the harness is ready).
