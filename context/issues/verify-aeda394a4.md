# Verify: the aeda394a4 deploy (web 1.18.15)

Deployed 2026-11-09. Surfer checks on live and marks pass or fail with what it saw. Anything new goes into this folder as a `reported` issue first.

Surfer: never open, move or resize windows; size checks use the same-origin frame in your one tab. Don't sign out.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 316 | Every side drawer (Change email, Add Filter, any other) stops above the footer at page end, wide and narrow (frame); docks ride inside the drawer | Surfer | | |
| 317 | Account: a pending email change shows its expiry as a tag, not loose text | Surfer | | |
| 318 | Account Pending row: Resend sends a link, toast plus a 60s countdown, a second press during it is ignored; refresh says "Not confirmed yet" while unconfirmed. Start a change to a test address if none is pending; don't confirm it | Surfer | | |
| 319 | Note only: on the first /account load after login, is the Pending row missing until a later load? (Don't sign out to test; record what you see) | Surfer | | |
| version | app.zwipe.net/version.json gives aeda394a4d69…; changelog shows 1.18.15 with its 3 entries | Surfer | | |
