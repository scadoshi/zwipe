# Zwiper Session Import Migration

## What

50 files in zwiper import `Session` through `zwipe::domain::auth::models::session::Session` inside `use zwipe::{...}` blocks. The `session.rs` proxy in zerver was deleted, so these must switch to `zwipe_core::domain::auth::models::session::Session`.

## Pattern

Every file has `auth::models::session::Session` somewhere inside a `use zwipe::{...}` block. The fix is:
1. Remove `auth::models::session::Session` from the `use zwipe::{...}` block
2. Add `use zwipe_core::domain::auth::models::session::Session;` as a separate import
3. If removing `Session` leaves the `zwipe::domain::` sub-block empty, clean up the empty branches

## Rules
- `Password` stays as `zwipe::domain::auth::models::password::Password` (server type)
- `Session` moves to `zwipe_core::domain::auth::models::session::Session`
- Keep all `zwipe::inbound::http::*` imports as-is (HTTP contract types stay in zerver)
- Don't change any logic, only imports
- Run `cargo clippy -p zwiper --all-targets` after to verify

## File Groups (for parallel agents)

### Group A: outbound/client/auth (5 files)
- `zwiper/src/lib/outbound/client/auth/login.rs`
- `zwiper/src/lib/outbound/client/auth/logout.rs`
- `zwiper/src/lib/outbound/client/auth/refresh.rs`
- `zwiper/src/lib/outbound/client/auth/register.rs`
- `zwiper/src/lib/outbound/client/auth/resend_verification.rs`

### Group B: outbound/client/card (7 files)
- `zwiper/src/lib/outbound/client/card/get_artists.rs`
- `zwiper/src/lib/outbound/client/card/get_card_types.rs`
- `zwiper/src/lib/outbound/client/card/get_card.rs`
- `zwiper/src/lib/outbound/client/card/get_keywords.rs`
- `zwiper/src/lib/outbound/client/card/get_languages.rs`
- `zwiper/src/lib/outbound/client/card/get_oracle_words.rs`
- `zwiper/src/lib/outbound/client/card/get_sets.rs`
- `zwiper/src/lib/outbound/client/card/search_cards.rs`

### Group C: outbound/client/deck + deck_card (8 files)
- `zwiper/src/lib/outbound/client/deck/create_deck.rs`
- `zwiper/src/lib/outbound/client/deck/delete_deck.rs`
- `zwiper/src/lib/outbound/client/deck/get_deck.rs`
- `zwiper/src/lib/outbound/client/deck/get_deck_profile.rs`
- `zwiper/src/lib/outbound/client/deck/get_deck_profiles.rs`
- `zwiper/src/lib/outbound/client/deck/get_deck_tokens.rs`
- `zwiper/src/lib/outbound/client/deck/update_deck_profile.rs`
- `zwiper/src/lib/outbound/client/deck_card/create_deck_card.rs`
- `zwiper/src/lib/outbound/client/deck_card/delete_deck_card.rs`
- `zwiper/src/lib/outbound/client/deck_card/import_deck_cards.rs`
- `zwiper/src/lib/outbound/client/deck_card/update_deck_card.rs`

### Group D: outbound/client/user (2 files)
- `zwiper/src/lib/outbound/client/user/change_password.rs`
- `zwiper/src/lib/outbound/client/user/delete_user.rs`

### Group E: inbound/screens (18 files)
- `zwiper/src/lib/inbound/screens/home.rs`
- `zwiper/src/lib/inbound/screens/auth/login.rs`
- `zwiper/src/lib/inbound/screens/auth/register.rs`
- `zwiper/src/lib/inbound/screens/profile/change_email.rs`
- `zwiper/src/lib/inbound/screens/profile/change_password.rs`
- `zwiper/src/lib/inbound/screens/profile/components/delete_account_dialog.rs`
- `zwiper/src/lib/inbound/screens/profile/components/email_verification.rs`
- `zwiper/src/lib/inbound/screens/deck/card/add.rs`
- `zwiper/src/lib/inbound/screens/deck/card/remove.rs`
- `zwiper/src/lib/inbound/screens/deck/card/view.rs`
- `zwiper/src/lib/inbound/screens/deck/card/filter/artist.rs`
- `zwiper/src/lib/inbound/screens/deck/card/filter/set.rs`
- `zwiper/src/lib/inbound/screens/deck/card/filter/oracle_text/keywords.rs`
- `zwiper/src/lib/inbound/screens/deck/card/filter/oracle_text/oracle_words.rs`
- `zwiper/src/lib/inbound/screens/deck/card/filter/types/other_types.rs`
- `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs`
- `zwiper/src/lib/inbound/screens/deck/components/deck_warnings.rs`
- `zwiper/src/lib/inbound/screens/deck/create.rs`
- `zwiper/src/lib/inbound/screens/deck/edit.rs`
- `zwiper/src/lib/inbound/screens/deck/export.rs`
- `zwiper/src/lib/inbound/screens/deck/import.rs`
- `zwiper/src/lib/inbound/screens/deck/list.rs`
- `zwiper/src/lib/inbound/screens/deck/view.rs`

### Group F: inbound/components (1 file)
- `zwiper/src/lib/inbound/components/auth/session_upkeep.rs`
