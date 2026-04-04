# Handler pub use → use Cleanup

## Context

Final step of the zerver pub import cleanup. After all domain proxy files are removed, 16 handler files still `pub use` HTTP contract types from zwipe-core. These should be downgraded to `use` (private) since the types don't need to be re-exported from the handler module.

**Prerequisite**: Card domain cleanup must be done first.

## The problem

16 handler files do `pub use zwipe_core::http::contracts::*::Http*;` which re-exports those types through zerver's module tree. 9 zwiper files import these types through the re-export path (`zwipe::inbound::http::handlers::*::Http*`).

## Step 1: Downgrade `pub use` → `use` in 16 handler files

```
zerver/src/lib/inbound/http/handlers/auth/authenticate_user.rs
zerver/src/lib/inbound/http/handlers/auth/change_email.rs
zerver/src/lib/inbound/http/handlers/auth/change_password.rs
zerver/src/lib/inbound/http/handlers/auth/change_username.rs
zerver/src/lib/inbound/http/handlers/auth/delete_user.rs
zerver/src/lib/inbound/http/handlers/auth/refresh_session.rs
zerver/src/lib/inbound/http/handlers/auth/register_user.rs
zerver/src/lib/inbound/http/handlers/auth/request_password_reset.rs
zerver/src/lib/inbound/http/handlers/auth/reset_password.rs
zerver/src/lib/inbound/http/handlers/auth/verify_email.rs
zerver/src/lib/inbound/http/handlers/deck/create_deck_profile.rs
zerver/src/lib/inbound/http/handlers/deck/update_deck_profile.rs
zerver/src/lib/inbound/http/handlers/deck_card/create_deck_card.rs
zerver/src/lib/inbound/http/handlers/deck_card/import_deck_cards.rs
zerver/src/lib/inbound/http/handlers/deck_card/update_deck_card.rs
zerver/src/lib/inbound/http/handlers/user/update_preferences.rs
```

Simple sed: `s/^pub use zwipe_core::http::contracts/use zwipe_core::http::contracts/`

## Step 2: Rewrite 9 zwiper files

These files import Http* types through `zwipe::inbound::http::handlers::*::Http*`. Change to `zwipe_core::http::contracts::*::Http*`.

Zwiper files that use these (find with `grep -rl 'zwipe::inbound::http::handlers' zwiper/src/`):
~9 files across screens and client modules.

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings
```
