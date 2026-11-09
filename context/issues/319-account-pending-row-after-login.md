# 319. Account shows no Pending row on the first visit after login

State: reported | Owner: - | PR: - | Build seen: local release 3a627a7c1 with #478

## Report
Scotland, 2026-11-09, web, while checking 318 in headless Chrome at 1280x600: a verified user with a pending email change (made through the API before logging in) opened Account right after logging in, and the Pending row was missing. The same user's next visit to Account showed it. The login response's user apparently lacks `pending_email`, and Account draws from the session's user until something refreshes it. The new Refresh button (318) sits on the Pending row, so it can't help while the row is missing.

## Repro
1. Create a verified user and request an email change for it (PATCH /api/user/change-email), so it has a pending address.
2. On the web, log in as that user.
3. Open Account: no Pending row.
4. Leave and open Account again: the Pending row shows the new address and its expiry tag.
