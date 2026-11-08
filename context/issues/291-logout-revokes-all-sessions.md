# 291. Logging out on one device logs out every device

State: approved | Owner: halo | PR: - | Build seen: 1e03520ef

## Report
Droid, 2026-11-08: signing out of the Android app ran zerver's revoke_sessions, which deletes every refresh token for the user, so the same account's web sessions (Droid's emulator Chrome, likely Surfer's) dropped too. Expected (to confirm with Scotty): Log out ends this device's session only; a separate 'Log out everywhere' would revoke all. The 10-session cap (raised from 5) suggests per-device sessions are intended.

## Decision
Scotty, 2026-11-08: sign out invalidates only that session, not every device.

## Plan
halo, 2026-11-08: web ends the refresh cookie's token family. The app gets an optional `family_id` claim in the access token (serde default), and logout ends that family. No client change. A token without the claim (old build, or minted before the deploy) falls back to revoking every session, as today. Composer approved: additive, no client change.
