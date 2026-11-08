# 291. Logging out on one device logs out every device

State: merged | Owner: halo | PR: #427 | Build seen: 1e03520ef

## Report
Droid, 2026-11-08: signing out of the Android app ran zerver's revoke_sessions, which deletes every refresh token for the user, so the same account's web sessions (Droid's emulator Chrome, likely Surfer's) dropped too. Expected (to confirm with Scotty): Log out ends this device's session only; a separate 'Log out everywhere' would revoke all. The 10-session cap (raised from 5) suggests per-device sessions are intended.

## Decision
Scotty, 2026-11-08: sign out invalidates only that session, not every device.

## Plan
halo, 2026-11-08: web ends the refresh cookie's token family. The app gets an optional `family_id` claim in the access token (serde default), and logout ends that family. No client change. A token without the claim (old build, or minted before the deploy) falls back to revoking every session, as today. Composer approved: additive, no client change.

## Fix
halo, #427: the access token carries an optional `family_id`; logout deletes only that family, scoped to the user. No claim means revoke all. Web cookie logout uses the refresh cookie's family, then the claim, then all. Password change, reset and delete still revoke every session. Tests: two sessions log out one (failed first), fallback, logout after refresh, browser logout leaves the app session. Verify after deploy: Droid signs in on the app and on phone web, logs out of one, the other stays signed in.
