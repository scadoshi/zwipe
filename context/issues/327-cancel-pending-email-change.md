# 327. A pending email change can't be cancelled

State: fixing | Owner: scotland | PR: #507 (server), #508 (client) | Build seen: release 049436b3d

## Report
Scotty, 2026-11-10, approved: Account's Pending row has Resend and Check but no way to cancel the change. There is no API route for it either, so a mistyped or unwanted change waits until its link expires (24h). Add Cancel on web and in the app.

327 plan (like 318): additive wire change. The server PR adds `POST /api/auth/cancel-email-change` and deploys first; the client PR then adds Cancel to the Pending row in the app and on the web through the shared screen model.

## Repro
1. Sign in, Account, change your email to a new address.
2. The Pending row shows the new address with Resend and Check, and nothing that drops the change.

## Fix
#507 (server, deploy first): `POST /api/auth/cancel-email-change` deletes the user's `email_change_requests` row, so its link stops confirming and the account keeps its address. 200 when an unexpired change was pending, 404 when none was (an expired row is deleted too and answers 404), 401 without a session. No rate limit, since it sends no mail. The route is in zwipe-core's paths and `FIXED_ROUTES`, with a `CancelEmailChange` endpoint and `ZwipeClient::cancel_email_change` for the client PR. Failing-first tests in `zerver/tests/user_flows.rs`: `cancel_email_change_drops_the_pending_change_and_kills_its_link`, `_without_a_pending_change_is_not_found`, `_twice_is_not_found_the_second_time`, `_needs_a_session`.

#508 (app and web, merges after #507 is live): the Pending row gets `Cancel` beside Resend and the check. It opens a confirm dialog ("Cancel email change", "Your email stays as it is. The link sent to the new address stops working.", `Cancel` / `Cancel change`), as Log out does. Confirming sends the cancel, clears the pending address from the session's user, toasts "Email change cancelled", and the row disappears. A failed cancel keeps the row. The zwipe-ui account model owns the action, the dialog's copy, and the cancel's in-flight state (failing-first tests: `the_pending_row_s_cancel_asks_first`, `a_confirmed_cancel_drops_the_pending_row`, `a_failed_cancel_keeps_the_pending_row`, `the_cancel_copy_reads_as_the_app_s`). The app performs the model's `Effect::CancelEmailChange`; the web takes the copy and dialog from the model, as its Pending row already does for Resend. Checked headless on the web against a local zerver with #507: Cancel, the dialog, confirm, the row gone and the server answering 404 afterward, at 1260 and 390, dark and light. The app isn't run on scotland. Verify: the Repro shows Cancel on the Pending row; confirming drops the row, and the old link no longer confirms.
