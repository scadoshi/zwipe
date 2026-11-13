# 427. zerver-admin has no way to inspect, sign out, re-verify, delete, lock or re-email an account

State: fixing | Owner: thinker | Type: feature | PR: - | Build seen: -

## Report
Composer for Scotty, 2026-11-12: more zerver-admin subcommands on the clap structure, all through the domain services, each writing an audit row; destructive ones confirm by typing the username (`--yes` skips) and support `--dry-run`: `show`, `lock`/`unlock`, `sign-out-all`, `resend-verification`, `delete`, `change-email`. Tests for each and a how-to in server.md.

## Fix
thinker, first part:
- `show <account>`: username, id, email, verified, pending change, created, last active, deck count and the latest 10 audit events (`UserService::fetch_account_summary`). Reads only; no audit row.
- `sign-out-all <account>`: revokes every refresh token (`revoke_sessions`, everywhere), audit `signed_out_everywhere`.
- `resend-verification <account>`: the app's resend (`send_verification_email`), refused once verified, audit `verification_email_sent`.
- `delete <account>`: the user's own delete without the password (`AuthService::admin_delete_user`). Audit rows cascade with the account, so the delete is logged (`admin_account_deleted`) instead.
- `sign-out-all` and `delete` ask for the username typed back; `--yes` skips, `--dry-run` reports and changes nothing.
- The account lookup is `AuthService::fetch_account(&AccountRef)`, shared with `rename`.

Second part, after Scotty's answers:
- Migration `20261112120000_account_lock_and_admin_action_log.sql`, additive only: nullable `users.locked_at` and `locked_reason` (NULL for every existing account, so nothing changes for it), and a new `admin_action_log` (account id, action, time; no foreign key, no username or email). No contract changes: old apps send and read what they did.
- `lock <account> [--reason]`: login answers a locked account with `AccountLocked`, the same 401 "invalid credentials" the wrong-password lockout gives; refresh answers 401 "invalid refresh token" as a revoked token does. `unlock` clears the stored lock; the in-memory lockout expires on its own. Audit `account_locked` / `account_unlocked`.
- `change-email <account> <new-email> [--reply-to]`: the taken check, the address switched at once and unverified, a pending change dropped, a verification email to the new address, and "Your Zwipe email was changed" to the old one with replies to the support address. Audit `email_changed`.
- Every command writes an `admin_action_log` row, `show` and `delete` included.
- The lookup has its own `FetchAccountError` ("no account named zatest1", "no account with email ..."). Errors print as one `error:` line with their causes and exit non-zero, never with a backtrace; a wrong confirmation is an error. The prompts read "Type zatest1 to sign it out everywhere:", "Type zatest1 to delete it:".

## Verification
- `cargo test -p zerver --bin zerver-admin` (parsing, the confirmation, the summary) and `--test admin_commands` (lookup, show, sign-out-all, resend-verification, delete, lock and unlock against login and refresh, change-email, the action log outliving a delete).
- On the server, the dummy-account lines in `operations/infrastructure/server.md` under zerver-admin.
