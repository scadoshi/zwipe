# 318. Pending email has no Refresh or Resend on web

State: approved | Owner: scotland | PR: - | Build seen: 1fd04e479

## Report
Scotty, 2026-11-09 (screenshot, web Account, Pending row): no way to check whether the new email was confirmed, or to resend the link. Add Refresh (re-fetch the account) and Resend (send a new verification link); match the mobile app if it already has them.

318 plan: option A (Scotty-approved additive wire change): new POST /api/auth/resend-email-change (204/404/429), server deploys before clients; app and web Pending rows get Refresh + Resend.
