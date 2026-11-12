# 423. Anyone can register Zwipe's own names, and names that differ only in case

State: fixing | Owner: thinker | Type: feature | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-12: reserve usernames (Scotty's names, the brand, the sites' routes, staff and system words, environments and infrastructure, business addresses), checked by zerver on register and on a username change, answered exactly as a taken name so older apps understand it and the list stays unread; no wire change. Existing accounts keep their names. A new name must also differ from every existing one in more than case (prod already holds pairs like scadoshi/Scadoshi and test/Test, which stay). Scotty claims a reserved name from the server only, with a tool that renames an existing account through the same path as a user's change minus the reserved check, records the audit row and mails the account a notice. First use: rename 'Test' to 'naxol56196'.

## Fix
thinker:
- zwipe-core `domain::user::models::reserved_username`: the list in one file, a group per comment for Scotty to trim, and `is_reserved`. A name matches after folding (lowercase; `.`, `-`, `_` and whitespace dropped; `0`→o, `1`/`l`→i, `3`→e, `4`/`@`→a, `5`/`$`→s, `7`→t); a word of four letters or more also matches with a number after it or official, real or the around it (zwipe1, realzwipe, thezwipe, zwipeofficial), so short words like me leave theme open.
- zerver: register answers a reserved name, or one another account holds in any case, with `UsernameTaken` (409, "username already taken"); a change answers either with `Duplicate` (409, the change's taken message). `username_taken` compares `lower(username)` and leaves the renaming account out; no constraint or index changes. An account may rewrite how its own reserved name is written.
- `zerver-admin rename <username|id|email> <new> [--no-notice] [--reply-to <address>]` (`zerver/src/bin/zerver-admin.rs`, `zwipe::inbound::admin`): the service's `admin_change_username` runs the user's rename path without the password or the reserved check, then the `username_changed` audit row, then the notice "Your Zwipe username has changed" (HTML in the account emails' style and plain text) through the email port, from `RESEND_EMAIL_FROM`. The email port gains an optional plain-text body and reply-to.
- Tests: zwipe-core normalization cases; `zerver/tests/reserved_usernames.rs` for the taken-shaped answers on register and change, case, grandfathered holders and pairs, and the admin rename (audit row, notice through the fake sender, by id, `--no-notice`, refusals). Smoke run of the binary against a local database: Test became naxol56196 with its audit row.

## Verification
- After deploy, on the server: build `zerver-admin` and rename Test (see the composer's report for the command).
