# 426. zerver-admin parses its own arguments and reads the account as a string at lookup

State: fixing | Owner: thinker | Type: feature | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-12: replace zerver-admin's hand-rolled argument parsing with clap (derive, a `rename` subcommand, room for more, `--help`); parse the account once into username, id or email, and `--reply-to` as a checked email; `--reply-to` defaults to the server's support address and `--help` says so. Tests for the parsing and for the rename paths not covered yet. Behavior otherwise unchanged.

## Fix
thinker:
- `zerver/src/bin/zerver-admin.rs`: a clap `Cli` with a `Command` enum (`Rename` today), `RenameArgs` with the account, the new name, `--no-notice` and `--reply-to <ADDRESS>`. `--reply-to` falls back to `SUPPORT_EMAIL_ADDRESS` from the server's config (support@zwipe.net when unset), and its help names that default.
- `AccountRef` (`zerver/src/lib/domain/auth/models/account_ref.rs`): a uuid is an id, a valid email is an email, anything else is a username, parsed once by clap. The service looks the account up by that kind only (a new `fetch_user_id_by_username` beside `fetch_user_id_by_email`). A username that is itself a valid email reads as an email; name that account by its id.
- The reply-to is an `Email` through `RenameNotice` and the auth port.
- clap 4.6 (derive) is a zerver dependency only.

## Verification
- `cargo test -p zerver --bin zerver-admin` (parsing: unknown flag, missing arguments, `--reply-to` without or with a bad address, a bad new name, help) and `--test reserved_usernames` (lookup by email, a missing email, a custom reply-to reaching the fake sender, the rename standing when the notice fails).
- On the server: `zerver-admin --help` and `zerver-admin rename --help` print the usage and the reply-to default.
