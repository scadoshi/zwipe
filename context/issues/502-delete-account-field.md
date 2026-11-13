# 502. Delete account's password field stretches across the dialog on a wide window

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer, 2026-11-13 (taken from laptop): the Delete account dialog should have a normal-size field.

## Before
Headless: at 1440 the dialog's password field is 902px wide across a 968px dialog; at 390 it is 254px in a 320px dialog, as the Account page's own fields are. Its font is 16px at both.

## Cause
`zwiper-web/assets/web.css`: on a wide window a dialog's field takes a field's width (`--web-field-max`, 32rem) centered under the message, by `.alert-dialog-description > .input`; a password field sits in its show-and-hide `.password-input-wrapper`, which the rule doesn't reach.

## Fix
scotland: the rule covers `.alert-dialog-description > .password-input-wrapper` too.

## Verification
- `delete_account.rs` test: the dialog field rule covers the password wrapper (red before).
- Headless: at 1440 the password field is 512px (32rem) centered in the 968px dialog (902px before); at 390 it is 254px, as before.
