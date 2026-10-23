# Verify-first registration

**Status: Phase 4 merged 2026-10-23 with the change-email half and PR #32's 409s, the floor at 1.13.0. Phases 1 and 2 shipped 2026-10-22 (PR #31, then 1.13.0 as iOS build 106 and Android vc68). After the deploy, the live check below is still owed. Companion: `confirm_first_email_change.md` (the change-email half).**

**As built:** on the `verify_first` path a taken username answers 409 "username already taken"; only clients that know 409 send the flag. The old path keeps its 422 until PR #32 (`duplicate-409`) merges with Phase 4. The zite verify page copy is unchanged: "You can close this page and go back to the app" already fits, since login works unverified. The client sends the user back to the Login screen it came from, with the username filled in, and toasts "Check your inbox to verify your email, then log in".

**Owner decisions, 2026-10-22, after the design below was written:** login stays open to unverified accounts (the recommendation under "What an unverified account can do" is accepted; a login gate was considered and dropped as stricter than a TCG app needs). The `account_exists` notice stays, because on register the person typing a taken address is often its owner. Every email on both branches is spawned off the request path so the response never waits on Resend; that replaces the "same work shape" parity argument below as the timing defense.

**One sentence:** `POST /api/auth/register` stops answering with a session, so a request for an address that already has an account can answer exactly like a request for a new one, and only the inbox learns which it was.

## Why

Register is the last place the API confirms that an email address has an account. Login equalizes its timing and answers 401 either way (`authenticate_user` in `zerver/src/lib/domain/auth/services.rs`), forgot-password returns 200 for unknown addresses (`request_password_reset`, same file), and change-email is getting the same treatment on the `email-enumeration` branch. Register still answers `422 "user with that username or email already exists"` (`From<RegisterUserError> for ApiError` in `zerver/src/lib/inbound/http/handlers/auth/register_user.rs`), rate limited to five per hour per IP (`register_config` in `zerver/src/lib/inbound/http/routes.rs`) but an oracle all the same. The 2026-10-10 audit left the decision open in `security_audit_followups.md` ("Email enumeration on register and change-email"); the owner made it on 2026-10-22: the register response goes flat and the happy path gets one step longer.

A flat response and a session cannot coexist. If a new address gets a session and a taken one does not, the body is the oracle; if a taken address got a session it would be someone else's account. So the session has to go from both branches, which is the whole cost: the user who just registered logs in instead of landing on Home.

The wire change needs a client floor. Other in-flight work needs one too (`sort_direction_wire.md` Phase 3), so this rides the same floor raise if the timing lines up.

## Current state

- **Handler.** `register_user` (`zerver/src/lib/inbound/http/handlers/auth/register_user.rs`) validates and hashes through `RegisterUser::new`, copies `platform` and `client_version` from the body, calls `auth_service.register_user`, spawns the metrics rows (`insert_lifetime_row`, `EventKind::Register`) and returns `201` with a `Session`. The duplicate arm is the `422` above.
- **Service.** `AuthService::register_user` (`zerver/src/lib/domain/auth/services.rs`) calls `create_user_and_refresh_token` (one transaction in `zerver/src/lib/outbound/sqlx/auth/mod.rs`: `INSERT INTO users`, then a refresh token that starts a family), mints the access JWT, sends the verification email fire-and-forget through `send_verification_email` (deletes old tokens, stores a SHA-256 hash with a 24-hour expiry, mails `email_templates/verify_email.html` with a `{web_base_url}/verify/{raw}` link) and returns the `Session`. `RegisterUserError::Duplicate` comes from `From<sqlx::Error>` in `zerver/src/lib/outbound/sqlx/auth/error.rs`, which maps any unique violation without saying which of the two unique columns (`users.email`, `users.username`, both from `20250810194439_create_users.sql`) fired.
- **Request type.** `RegisterUser` in `zerver/src/lib/domain/auth/requests/register_user.rs`; its module doc draws the flow ending in "Create Session" and "Return Session to Client".
- **Contract.** `HttpRegisterUser` in `zwipe-core/src/http/contracts/auth.rs` carries `username`, `email`, `password` and the additive `platform` and `client_version`, whose doc says they are "recorded on the auto-login session". The typed endpoint `Register` in `zwipe-core/src/http/endpoints/auth.rs` has `type Response = Session`, and `zwipe-client/src/call.rs` treats every 2xx as success and decodes the body into that type, so a 1.12.x client that received any other body on this path would fail with `ClientError::Decode`.
- **Routes.** `REGISTER_ROUTE`, `VERIFY_EMAIL_ROUTE` (public) and `RESEND_VERIFICATION_ROUTE` (authed, burst 1 then one per 60 seconds per user, matching the client's countdown) in `zwipe-core/src/http/paths.rs`; mounted in `routes.rs`. `verify_email` (`handlers/auth/verify_email.rs`) takes the raw token, marks `email_verified_at`, returns 200 with no body. `resend_verification` (`handlers/auth/resend_verification.rs`) is 422 once verified.
- **What `email_verified_at` gates.** Nothing is refused outright. An unverified account logs in, browses and builds, with caps: `UNVERIFIED_MAX_DECKS_PER_USER = 1` and `UNVERIFIED_MAX_CARDS_PER_DECK = 100` against 20 and 500 (`zerver/src/lib/domain/deck/mod.rs`). The handlers that enforce them re-read the user row (`create_deck_profile`, `clone_deck`, `create_deck_card`, `import_deck_cards`, `import_archidekt`) because the JWT claims (`UserClaims` in `zwipe-core/src/domain/auth/models/access_token.rs`) do not carry the flag. The integration test helper `TestApp::verify_email` exists for exactly that (`zerver/tests/common/mod.rs`).
- **App.** `zwiper/src/lib/inbound/screens/auth/register.rs` calls `auth_client().register(request)`, saves the session, sets the `session` signal and pushes `Router::Home {}`. Home (`screens/home.rs`) refreshes the user on mount, toasts "Verify your email!" while `email_verified_at` is `None`, greets once verified, and opens the `HINT_FIRST_LOGIN` dialog through `use_one_time_hint` (a `hints_shown` key on the user, `zwipe-core/src/domain/user/models/hints.rs`). Profile (`screens/profile/mod.rs`) shows a Verification row with `VerificationActions` (`profile/components/email_verification.rs`: Resend with the 60-second cooldown, and a check button that re-fetches the user). Login (`screens/auth/login.rs`) does the same save-and-push on success. There is no deep link or universal link into the app (no `applinks`, `assetlinks` or custom scheme anywhere in `zwiper/` or `zite/`).
- **Site.** zite's `Verify` page (`zite/src/pages/verify.rs`, route `/verify/:token` in `zite/src/main.rs`) posts the token and says "You can close this page and go back to the app"; on failure it points at Resend on Profile. zite holds no session and deploys with the server on every push to main.
- **Tests.** `zerver/tests/common/mod.rs` has `FakeEmailSender` (`last_body`, `last_token("verify")`), `TestApp::register` (asserts 201, returns the access token and user id; most suites build on it), `TestApp::verify_email`. Flows in `zerver/tests/auth_flows.rs`, token edges in `auth_edges.rs` (`verify_email_via_captured_token`), `register_stamps_client_version` in `auth_session_version.rs`, and `user_flows.rs` for the profile changes. `route_coverage.rs` checks the path table in `paths.rs`.
- **Floor.** `MIN_CLIENT_VERSION` in `zerver/src/lib/config.rs` (default `0.0.0`, refuses startup if malformed) is served by `GET /api/client/min-version` and enforced only by the app's own "Update required" screen (`session_upkeep.rs`). The server never rejects a request for being below the floor. `client_version` reaches the server as a body field on login, register and refresh (`HttpAuthenticateUser`, `HttpRegisterUser`, `HttpRefreshSession`), not as a header.
- **Companion.** The `email-enumeration` branch (`5d0bbb20`) made `change_email` return `ChangeEmailOutcome::{Changed, Taken}` and mailed the owner `email_claim_attempted.html`. Testing showed the response body still differed, so it is being replaced by `confirm_first_email_change.md`, which drops the owner email. This plan borrows the outcome-enum shape and the template style only.

## Design

### Register semantics

`POST /api/auth/register` creates the account unverified and answers `202 Accepted` with a body that carries no session, on both branches:

```json
{ "status": "verification_sent" }
```

New contract type in `zwipe-core/src/http/contracts/auth.rs`: `HttpRegistrationAccepted { status: RegistrationStatus }` with `RegistrationStatus::VerificationSent` serialized as `"verification_sent"`. An enum rather than a bare string so a second outcome can be added later without the client matching on text; one variant today. 202 rather than 201 because the server has not committed to an account from the caller's point of view, and because a distinct status keeps a 201-expecting old client from ever confusing the two bodies.

Branches, in `AuthService`, as `RegisterOutcome::{Created(User), Taken}` the way the companion returns `ChangeEmailOutcome`:

- **Username taken.** Checked first, by a `SELECT` on `users.username` before any insert, and answered inline with the existing `422`. Usernames are public, the audit note already carves this out, and checking it first means the insert's unique violation can only ever be the email (bar a race, which also lands on the email arm and is harmless).
- **New email.** `INSERT INTO users` without a refresh token (the transaction in `create_user_and_refresh_token` loses its second half; see Phase 1 for how the two paths coexist), then `send_verification_email` as today. Metrics as today, on the handler side, from `Created(user)`.
- **Taken email.** The insert's unique violation (`DuplicateEmail`, split from today's `Duplicate` in `zerver/src/lib/outbound/sqlx/auth/error.rs` by constraint name). Nothing is written. The existing owner is sent `email_templates/account_exists.html`, new, in the same style as `email_claim_attempted.html` on the companion branch: "Someone tried to create a Zwipe account with this email address. Nothing changed: you already have one. Log in with the app; if you have forgotten the password, Forgot password on the log in screen resets it. If this wasn't you, no action is needed," plus the Discord and `{support_email}` footer. Subject "You already have a Zwipe account". Sent with `send_change_notification` (fire-and-forget, logged on failure). The email is sent to the address as given, since the unique index just confirmed it is on a row.

Timing: both branches spawn their email and answer without waiting on Resend, so the round trip that would otherwise separate them never reaches the response. What's left is the Argon2 hash (done in the handler before the service runs), one insert attempt, and the token write on the created branch, which is one small query. The per-IP limit (five per hour) blunts what remains.

Notice cooldown: one `account_exists` notice per address per hour, kept in memory beside `LoginLockout` (`zerver/src/lib/domain/auth/lockout.rs` is the pattern: a keyed map with expiry, no migration). Without it, five IPs can send a victim five emails an hour each. The cooldown check runs on the taken branch only and is cheap. The handler still answers 202 when the notice is suppressed.

Logging: `tracing::info!(event = "register")` stays on both branches; the taken branch adds `event = "register_existing_email", user_id` of the existing account. Server logs are private; the response is not.

Rate limit: unchanged. `register_config` stays at five per hour per IP.

### What an unverified account can do

**Recommendation: login works for an unverified account, exactly as today, and nothing new is refused.** The register screen hands the user to Login instead of Home; after that the app is the app they already have, caps and nudges included.

Why this and not "login refused until verified":

- The oracle is closed by the register response alone. Login already hides whether an account exists; what an unverified session can do has no bearing on enumeration.
- The unverified mode already exists end to end: the one-deck and 100-card caps on the server, the Home nudge, the Profile Verification row with Resend and check. A hard gate would throw that away for no security gain.
- `resend_verification` is authed and keyed by user id. Refusing login until verified would force an unauthenticated resend keyed by email, which is one more place to answer flat and one more thing the companion work would have to cover. A user whose first email went to spam would otherwise have no way back in.
- Refusing login means a new `403` on `authenticate_user`, a new client error arm, and a stranded user if the mail is slow. Allowing it means zero changes to login.

Endpoints affected by this choice: none change. For the record, the ones that read `email_verified_at` and would be the gate if the owner ever wants one: `POST /api/deck` (`create_deck_profile`), `POST /api/deck/{id}/clone` (`clone_deck`), `POST /api/deck/{id}/card` (`create_deck_card`), the two import routes (`import_deck_cards`, `import_archidekt`), and `POST /api/auth/resend-verification`, which stays the way back for anyone whose link expired.

`hints_shown` and first run: unchanged. `HINT_FIRST_LOGIN` opens on Home the first time the account lands there, which is now after the first login rather than after register. The "Verify your email!" toast fires on that same mount if the user logged in before clicking the link, and the Profile row shows Resend. Nothing in the hint flow keys on how the session was obtained.

### Verification completes the flow

**Recommendation: the verify page tells the user to open the app and log in. It does not hand out a session.**

zite is a static site with no session store, the link is opened in whatever browser the mail client picks, and there is no deep link or universal link into zwiper to carry a token across (none exists today; adding one is its own project with App Site Association and `assetlinks.json` on zwipe.net). Minting a session for the browser page would put a bearer token somewhere nothing can use it. Mail scanners also follow links, and a scanner that collects a session is worse than one that burns a verification token.

So `POST /api/auth/verify-email` keeps returning 200 with no body. The zite `Verify` page's success copy changes from "You can close this page and go back to the app" to "Email verified. Open Zwipe and log in." The failure copy ("Tap Resend on Profile in the app for a new link") stays right, because login works unverified. zite deploys with the server, so this copy can change in Phase 1.

In the app, the register screen after a 202 navigates to Login with the email prefilled and one info toast: "Check your inbox to verify your email, then log in." The same screen and the same toast on both branches, since the screen cannot know which it was. The owner of a taken address finds the notice in their inbox; a user who typo'd their address sees nothing arrive and registers again, which costs them nothing (no account was made with the typo either way, as far as they can tell).

Not doing: a "waiting for verification" screen that polls. Login works unverified, so there is nothing to wait for.

### The wire, and why a field rather than a route or a version check

Phase 1 needs the server to serve both behaviors at once, keyed on something the new client sends and the old one does not. Three candidates:

- **A new request field.** `HttpRegisterUser` gains `#[serde(default)] verify_first: bool`. Absent means the behavior that shipped before the field existed (auto-login, 201), which is the rule in `api_evolution.md` verbatim. Phase 4 deletes the field and the old arm; the path never changes.
- **A new route.** `POST /api/auth/register-verify-first` or similar. Clean while both exist, but Phase 4 leaves either an odd permanent name or a second rename dance, and `FIXED_ROUTES` in `paths.rs` plus `route_coverage.rs` churn twice.
- **The `client_version` body field.** Serve 202 to versions at or above the Phase 2 cut. Needs that number baked into the server before the client exists, misfires on a client built from main between the two, and treats a telemetry field as a contract switch.

**Recommendation: the field.** It is spoofable by design (an attacker omits it and gets the old 422), which is why the oracle only closes at Phase 4; the choice of switch does not move that date, so the simplest switch wins.

The typed endpoint needs two types while both bodies exist, because `Endpoint::Response` is fixed per type: `Register` keeps `Response = Session`, and a new `RegisterVerifyFirst(pub HttpRegisterUser)` on the same path has `Response = HttpRegistrationAccepted` and sets `verify_first: true` in its body. zwipe-client gets `register_verify_first`. Phase 4 deletes the old type and renames the new one to `Register`.

## Phase 1: server serves both (zwipe-core, zerver, zite)

One commit, deployed before any client build that sends the field.

- `zwipe-core/src/http/contracts/auth.rs`: `verify_first: bool` with `#[serde(default, skip_serializing_if = "std::ops::Not::not")]` on `HttpRegisterUser`; `HttpRegistrationAccepted` and `RegistrationStatus`. Update the `HttpRegisterUser` doc (the session is only minted on the old path). Serde tests beside the existing ones: a body without `verify_first` reads `false`; the accepted body serializes to the literal `{"status":"verification_sent"}`.
- `zwipe-core/src/http/endpoints/auth.rs`: `RegisterVerifyFirst`. `zwipe-client/src/auth/register.rs`: `register_verify_first`.
- `zerver/src/lib/outbound/sqlx/auth/error.rs`: `Duplicate` splits into `DuplicateUsername` and `DuplicateEmail` on the constraint name (`users_username_key`, `users_email_key`); the `From<sqlx::Error>` arm reads `DatabaseError::constraint()`. `zerver/src/lib/outbound/sqlx/auth/mod.rs`: `username_exists(&Username)`, and `create_user(&RegisterUser) -> User` (the insert alone); `create_user_and_refresh_token` stays for the old path until Phase 4.
- `zerver/src/lib/domain/auth/requests/register_user.rs`: `RegisterOutcome`, the two duplicate variants; module doc gets the second flow.
- `zerver/src/lib/domain/auth/services.rs`: `register_user_verify_first(&RegisterUser) -> Result<RegisterOutcome, RegisterUserError>` beside the existing `register_user`; the cooldown map; `email_templates/account_exists.html`. `ports.rs` gains the method and the two repository calls.
- `zerver/src/lib/inbound/http/handlers/auth/register_user.rs`: branch on `body.verify_first`. The new arm returns `(StatusCode::ACCEPTED, Json(HttpRegistrationAccepted { .. }))` for both outcomes and spawns metrics only for `Created`. The `DuplicateUsername` arm maps to the existing 422 text; `DuplicateEmail` never reaches `ApiError` on the new path, and on the old path both map to today's 422 so a 1.12.x client sees no change at all.
- `zite/src/pages/verify.rs`: the success copy above.
- Changelog: none yet; the user-visible change ships with the client in Phase 2.

Tests, Phase 1 (`zerver/tests/auth_flows.rs`, or a new `auth_register.rs` if it grows past four cases; pattern as the existing suites, `TestApp` plus `FakeEmailSender`):

- `register_without_flag_still_auto_logs_in`: covered by `register_authed_login_refresh` as it stands; leave it.
- `register_verify_first_new_email_is_202_without_session`: 202, body equals the literal above, no `access_token` key, `emails.last_token("verify")` is `Some`, then `LOGIN_ROUTE` with the same credentials is 200 and `VERIFY_EMAIL_ROUTE` with the captured token is 200.
- `register_verify_first_taken_email_matches_new`: register alice (old path), then verify-first with a fresh username and alice's address: 202 and a body byte-equal to the new-address case; `emails.last_body()` contains the account-exists subject line and no `/verify/`; `SELECT count(*) FROM users` unchanged; alice's password still logs in.
- `register_verify_first_taken_username_is_422`.
- `register_taken_email_notice_is_on_cooldown`: two taken attempts in a row send one email.
- `TestApp::register_pending(username) -> String` (the email) in `common/mod.rs` for these; `TestApp::register` is untouched until Phase 4.
- `auth_session_version.rs` `register_stamps_client_version` keeps passing: the old path still stamps.

Verification: `cargo +nightly fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p zwipe-core -p zerver` with `DATABASE_URL` from `zerver/.env`, `cargo sqlx prepare --workspace` if the new queries use the macros. After deploy: a 1.12.x client registers and lands on Home as before; a `curl` with `"verify_first": true` to `https://api.zwipe.net/api/auth/register` returns 202 with the literal body for a fresh address and for a taken one.

## Phase 2: the client

The next zwiper cut after Phase 1 is deployed. Minor bump per `versioning.md` (a behavior the user sees), so 1.13.0 unless a minor line is already open.

- `zwiper/src/lib/inbound/screens/auth/register.rs`: call `register_verify_first`; on `Ok` push `Router::Login {}`, prefill the identifier, show the toast. Prefill travels through a small signal provided at the root beside `session` (a `Signal<Option<String>>` the login screen reads once and clears), not a route parameter, so the email never lands in a URL or a log. Copy per `context/development/ui_text_conventions.md`.
- `zwiper/src/lib/inbound/screens/auth/login.rs`: read the prefill.
- Changelog entry (one phrase, no period): "Registering sends a verification email and takes you to log in".
- The anonymous funnel (`RegisterSubmitted` in `telemetry/vocabulary.rs`, `user_events.register` on the server) gains a step between submit and first session; `todo.md`'s funnel read should count first logins of new accounts rather than register events once this ships. No new event kinds.

Manual check on the app, against the dev server (debug builds log the verification link, `dev_email_verification` in `services.rs`):

1. Register with a new address: Login screen, email prefilled, one toast. Verify link in the log; open it in a browser: zite says open the app and log in. Log in: Home greets with the first-login hint.
2. Register with a new address and log in without verifying: Home toasts "Verify your email!", Profile shows Resend, deck creation stops at one deck.
3. Register with an address that already has an account and any password: identical screen and toast; the server log shows `register_existing_email`; the inbox (or the fake sender in a test) gets the account-exists notice and nothing else.
4. Register with a taken username: the inline 422 as today.

Record the version here when it is cut: **client sends `verify_first` from 1.13.0** (iOS build 106, Android vc68, submitted 2026-10-22).

## Phase 3: raise the floor and wait

When the Phase 2 build is live on both stores and rolled out, set `MIN_CLIENT_VERSION` to that version in `/etc/zwipe/zerver.env` and restart zerver, the way `access_token_window.md` did on 2026-10-13 for 1.11.2. Confirm `GET https://api.zwipe.net/api/client/min-version` returns it. Any client below it gets the update screen. Wait a few quiet days and watch `zcripts/metrics/errors.sql`.

This phase changes nothing about the oracle. The floor is enforced by the app, not the server, so a scripted caller still sends a body without `verify_first` and still gets the old 422. Phase 3 exists so that Phase 4 strands nobody who is still using the app.

## Phase 4: remove the auto-login path

**Merged 2026-10-23.** Register answers only 202. `HttpRegisterUser` is down to username, email and password, and serde ignores what 1.13.x still sends (no contract uses `deny_unknown_fields`). `RegisterVerifyFirst` became `Register` and the client's `register_verify_first` became `register`. `AuthService::register_user` is the verify-first body now. `create_user_and_refresh_token` is gone, along with its `.sqlx` entry, and so are `RegisterUserError::{Duplicate, FailedAccessToken, CreateSessionError}` and `RegisterUser`'s `platform` and `client_version`. The variants kept their as-built names, `DuplicateEmail` and `UsernameTaken`, and any unique violation other than the email maps to `UsernameTaken`. `TestApp::register` registers, waits for the verification email and logs in from its own address so it doesn't use up the test's five logins. `register_stamps_client_version` is gone; the two refresh cases now take their version from a login. New tests `retired_fields_are_ignored` and `no_flag_no_session`. PR #32 came in as a cherry-pick, so duplicates answer 409, except a taken email on register, which still answers 202. The handler's `DuplicateEmail` arm can't be reached (the service turns it into `EmailTaken`) and answers a bare 500 in case it ever is, so it can't name the address.

One commit, after Phase 3 has been quiet. The oracle closes here, when every caller goes through the new path.

- `HttpRegisterUser`: delete `verify_first`. `platform` and `client_version` have nothing left to stamp, so delete them too and the `register_deserializes_without_client_version` test with them; a 1.12.x body still deserializes (serde ignores unknown fields) but is below the floor anyway.
- Endpoints: delete `Register`, rename `RegisterVerifyFirst` to `Register`; zwipe-client `register` returns `HttpRegistrationAccepted`; `register_verify_first` goes.
- zerver: the handler keeps only the 202 arm; `AuthService::register_user` is the verify-first body; `create_user_and_refresh_token` goes, `RegisterUserError` drops `FailedAccessToken` and `CreateSessionError`; the `register_user.rs` module doc shows one flow.
- Tests: `TestApp::register` becomes register-then-login (and most suites call `verify_email` after it already); delete `register_stamps_client_version` (`login_stamps_client_version` and the refresh cases stay); delete the old-path assertion in `register_authed_login_refresh` and make it read `202` then login.
- Grep gate: `auto-login` and `CREATED` appear nowhere under `handlers/auth`; `Duplicate` is only `DuplicateUsername` and `DuplicateEmail`.
- Docs: `context/plans/security_audit_followups.md` ticks the register half; this file moves to `archive/` with its status line updated.

Verification: the Phase 1 suite minus the deleted cases is the proof; clippy and fmt as always; deploy; one live registration with a new address and one with a taken one from the shipped app, both landing on Login the same way.

## Decided

1. **No login right after the 202.** The app fills the username into the Login screen instead (`68cfa499`), and the user logs in once they've verified. Chaining a login wouldn't reopen the oracle, since login only confirms a full credential pair, so this stays a one-arm change in `register.rs` if the funnel ever argues for it.
2. **The per-address cooldown on the account-exists notice shipped in Phase 1.** Without it the per-IP limit was the only cap on how often someone else's inbox could get the notice.
3. **Taken usernames stay inline.** Usernames are public, so saying one is taken tells nobody anything. It answers 409 since Phase 4, not 422.
