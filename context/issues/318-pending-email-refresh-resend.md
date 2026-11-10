# 318. Pending email has no Refresh or Resend on web

State: verified | Owner: scotland | PR: #477, #478 | Build seen: 1fd04e479

## Report
Scotty, 2026-11-09 (screenshot, web Account, Pending row): no way to check whether the new email was confirmed, or to resend the link. Add Refresh (re-fetch the account) and Resend (send a new verification link); match the mobile app if it already has them.

318 plan: option A (Scotty-approved additive wire change): new POST /api/auth/resend-email-change (204/404/429), server deploys before clients; app and web Pending rows get Refresh + Resend.

## Fix
#477 (server, deployed first): `POST /api/auth/resend-email-change` mails a fresh link for a pending change, 404 with none pending, 429 within a minute. #478 (app and web): the Pending row gets Resend, with its own 60s cooldown, and a refresh that says whether the change went through. Tests in `zerver/tests/user_flows.rs` and the zwipe-ui account model. Shipped in 1.18.15. The refresh button's missing accessible name was 321.

## Verification
Surfer, live aeda394a4 ([`verify-aeda394a4.md`](verify-aeda394a4.md)): PASS. Refresh toasts "Not confirmed yet. Check test+pending@zwipe.net"; Resend sends one request, toasts, counts down from 59s, ignores a second press, and comes back after 60s.
