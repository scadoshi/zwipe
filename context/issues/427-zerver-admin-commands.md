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

Open, waiting on Scotty: `lock`/`unlock` (no stored lock exists; needs a migration and a sign-in check) and `change-email` (pending link or immediate switch).

## Verification
- `cargo test -p zerver --bin zerver-admin` (parsing, the confirmation, the summary) and `--test admin_commands` (lookup, show, sign-out-all, resend-verification, delete).
- On the server, the dummy-account lines in `operations/infrastructure/server.md` under zerver-admin.
