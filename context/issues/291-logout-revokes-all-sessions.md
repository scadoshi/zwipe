# 291. Logging out on one device logs out every device

State: approved | Owner: halo | PR: - | Build seen: 1e03520ef

## Report
Droid, 2026-11-08: signing out of the Android app ran zerver's revoke_sessions, which deletes every refresh token for the user, so the same account's web sessions (Droid's emulator Chrome, likely Surfer's) dropped too. Expected (to confirm with Scotty): Log out ends this device's session only; a separate 'Log out everywhere' would revoke all. The 10-session cap (raised from 5) suggests per-device sessions are intended.

## Decision
Scotty, 2026-11-08: sign out invalidates only that session, not every device.
