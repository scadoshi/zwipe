# Confirm-first email change

**Status: Phase 1 deployed 2026-10-22 (PR #31) and tested live, Phase 2 submitted as 1.13.0 (iOS build 106, Android vc68) the same day. Next is Phase 3, the floor at 1.13.0. Replaces the first approach (commit `5d0bbb20`, kept in the branch history and undone by the next one). The Phase 4 move of the remaining duplicates to 409 waits in PR #32 (`duplicate-409`). Companion: `verify_first_registration.md`.**

**One sentence:** changing your email stops changing it; the new address is held as pending until someone clicks a link sent to it, so a free address and a taken one get the same response and leave the account looking the same.

## Why the branch's approach doesn't close the leak

`5d0bbb20` makes `change_email` answer 200 for a taken address, but the body is the caller's user with the old email, while a real change returns the user with the new one. The app's toast reads the email from the response ("Email changed to {email}"), so it shows the leak on screen: tested 2026-10-22 on a local server, `asdf` asking for `t@t.com` (owned by `test`) got "Email changed to a@a.com". Echoing the requested address back instead doesn't help either. The next profile fetch shows the old email again, and the flip back gives the same answer a moment later. Any response is a leak as long as the account changes in one case and not the other.

So the account doesn't change in either case until the new address proves it can receive mail.

## Decisions (owner, 2026-10-22)

- Pending model as below.
- A taken address sends nothing to anyone. The person typing someone else's address is never that address's owner, and the owner can't act on a warning. `email_claim_attempted.html` from the branch goes.
- Emails leave the request path. Sending one is a few hundred milliseconds of Resend round trip; if the free branch waits on it and the taken branch sends nothing, the response time is the oracle. Both branches spawn their send (or their non-send) and answer without waiting.
- If the address gets taken between request and click, the confirm step fails (409) and tells whoever clicked. Only someone holding that inbox can see that.

## Design

### Storage

New table, one pending change per user:

```sql
CREATE TABLE email_change_requests (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    new_email TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Separate from `email_verification_tokens` because `send_verification_email` starts with `delete_email_verification_tokens(user_id)`: a Resend on Profile would otherwise wipe a pending change. No unique constraint on `new_email`; uniqueness is decided once, on `users.email`, at confirm time.

### Request: `POST /api/auth/change-email`

Same path, same body (`HttpChangeEmail`: email and password), password still re-verified. Then, on both branches:

1. Generate a token and upsert the `email_change_requests` row (a new request replaces an old one). 24-hour expiry, like verification.
2. Check whether `users.email` already has the address.
3. Free: spawn the send of `confirm_email_change.html` with a `{web_base_url}/confirm-email/{raw}` link. Taken: spawn nothing, log `event = "email_change_taken"` with the user id. The debug-build `dev_email_verification`-style log of the link happens on the free branch only.
4. Answer 200 with the caller's user, `email` unchanged, `pending_email` set.

The writes are identical on both branches, the token row exists either way (a taken one is simply never mailed), and the response doesn't wait on Resend. Audit: `EmailChangeRequested` on both branches; `EmailClaimAttempted` goes.

`User` gains `#[serde(default)] pending_email: Option<Email>`, filled from an unexpired `email_change_requests` row wherever the user is read for the client (the change-email response, `GET` user). Additive, so a 1.12.x client decodes it and ignores it.

### Confirm: `POST /api/auth/confirm-email-change`

Public, takes the raw token, like `verify_email`. In one transaction: delete the row by hash where unexpired and return `user_id` and `new_email`; `UPDATE users SET email = new_email, email_verified_at = NOW()`. Then send `email_changed.html` to the old address (moved here from request time, since this is when the change happens) and write `EmailChanged`.

A unique violation on the update means someone registered or switched to the address since the request. Answer 409 "that address is already in use by another account". The update runs in a savepoint so the row's deletion still commits and the link is dead.

Expired or unknown token: 422 "token not found or expired", like `verify_email`. `ApiError::Conflict` and `ClientError::Conflict` were added for the 409; it is safe here because only zite calls this route. Existing routes keep 422 until the floor passes the client that knows 409.

### zite

New page `zite/src/pages/confirm_email.rs`, route `/confirm-email/:token`, built like `verify.rs`. Success: "Email changed. Open Zwipe to keep going." The taken and expired cases get their own copy; both say to request the change again from Profile.

### App

`zwiper/src/lib/inbound/screens/profile/change_email.rs`: on success, toast "Check {typed address} for a link to confirm the change." The typed address, not the response's, since the response deliberately doesn't say. Session takes the returned user as now.

Profile's Email row shows `pending_email` under the current address while there is one, with the time it expires. Verification state of the current address is untouched by a request, so the "Verify it to restore full access" path and the unverified caps no longer apply to email changes.

## The wire and old clients

A 1.12.x client calling the new server would get a 200 with its old email, toast "Email changed to {old email}", and receive a confirmation email it doesn't explain. Not broken, but confusing. Two ways:

- **Ship the server change with the next client and accept the confusion** for anyone on 1.12.x who changes their email before updating. Email changes are rare.
- **Gate on a request field**, `#[serde(default)] confirm_first: bool` on `HttpChangeEmail`, the same way `verify_first_registration.md` gates register, and drop the old path after the floor raise both plans share.

Recommendation: the field. It's one boolean, the floor raise is already planned for registration, and the two plans then retire their old paths in the same commit.

## Phases

**Phase 1, server (zwipe-core, zerver, zite).** Migration; `pending_email` on `User`; `confirm_first` on `HttpChangeEmail`; `ConfirmEmailChange` endpoint type and route in `paths.rs`; repository calls for the upsert, the pending read, and the confirm transaction; the service and handler arms; the two templates; the zite page. The old arm (no field) behaves like main, not like `5d0bbb20`. `cargo sqlx prepare --workspace` after the queries.

Tests in `zerver/tests/user_flows.rs`, replacing `change_email_to_taken_address_answers_like_success_and_tells_owner`:

- free and taken addresses get byte-equal bodies (same `email`, same `pending_email`);
- free sends exactly one email, to the new address, and the captured token confirms; after confirm, `email` is the new address and verified, and the old address got `email_changed`;
- taken sends no email and the owner's row is untouched;
- a second request replaces the first, and the first token no longer confirms;
- confirm after someone else took the address is 409 and leaves both rows as they were;
- a Resend verification during a pending change leaves the change row alone.

**Phase 2, client.** The toast, the pending row on Profile, `confirm_first: true`. Changelog: "Changing your email takes effect once you confirm it from the new address".

**Phase 3, floor.** Shared with `verify_first_registration.md` Phase 3.

**Also behind the floor: the `duplicate-409` branch.** On top of `account-enumeration`, it moves the seven duplicate answers (register, change username, change email's old path, create, rename and clone deck, add card) from 422 to 409. A client from before `ClientError::Conflict` shows a generic error for 409, so it merges with Phase 4, not before. Messages are unchanged and the app shows them as before.

**Phase 4, cleanup.** Delete the field and the old arm, with registration's Phase 4. Tick the change-email half in `security_audit_followups.md` and archive this file.

## What happens to the branch

Done 2026-10-22: the `5d0bbb20` approach was replaced in the commit that built Phases 1 and 2. `ChangeEmailOutcome`, `email_claim_attempted.html` and `EmailClaimAttempted` are gone; its test became the starting shape for the new ones.
