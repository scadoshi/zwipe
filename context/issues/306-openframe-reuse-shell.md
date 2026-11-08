# 306. About and Privacy draw a second app frame when signed in

State: fixing | Owner: halo | PR: #444 | Build seen: 1d7053e88

## Report
From 299's cause (scotland): /about and /privacy sit under OpenFrame, which for a signed-in user draws its own Shell, so a second header, nav and LeaveDialog exist while those pages are open. 299 fixes the symptom (a dropping dialog clears only its own slots). Scotty, 2026-11-08: OK to clean up so there is only ever one Shell.

## Fix direction
For a signed-in user, About and Privacy render inside the one authed Shell (no second Shell or LeaveDialog); signed out they keep the open frame. Start after scotland's 299 PR merges, on top of it.
