# 327. A pending email change can't be cancelled

State: fixing | Owner: scotland | PR: #507 (server), client to follow | Build seen: release 049436b3d

## Report
Scotty, 2026-11-10, approved: Account's Pending row has Resend and Check but no way to cancel the change. There is no API route for it either, so a mistyped or unwanted change waits until its link expires (24h). Add Cancel on web and in the app.

327 plan (like 318): additive wire change. The server PR adds `POST /api/auth/cancel-email-change` and deploys first; the client PR then adds Cancel to the Pending row in the app and on the web through the shared screen model.

## Repro
1. Sign in, Account, change your email to a new address.
2. The Pending row shows the new address with Resend and Check, and nothing that drops the change.

## Fix
#507 (server, deploy first): `POST /api/auth/cancel-email-change` deletes the user's `email_change_requests` row, so its link stops confirming and the account keeps its address. 200 when an unexpired change was pending, 404 when none was (an expired row is deleted too and answers 404), 401 without a session. No rate limit, since it sends no mail. The route is in zwipe-core's paths and `FIXED_ROUTES`, with a `CancelEmailChange` endpoint and `ZwipeClient::cancel_email_change` for the client PR. Failing-first tests in `zerver/tests/user_flows.rs`: `cancel_email_change_drops_the_pending_change_and_kills_its_link`, `_without_a_pending_change_is_not_found`, `_twice_is_not_found_the_second_time`, `_needs_a_session`.
