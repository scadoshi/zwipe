# 299. Create deck's leave guard stops working after visiting About

State: reproduced | Owner: - | PR: - | Build seen: 1d7053e88

## Report
Surfer, live web 1d7053e88, wide, 2026-11-08. Once About has been opened in the session (full load or in-app), Create deck with unsaved edits can't be left. Header Back, the Decks nav link, the logo and browser Back all do nothing, no "Leave without saving?" dialog shows, and the console has no errors. This is what Scotty saw in 281, which was closed as can't-reproduce because my earlier runs never went through About.

## Repro
1. Load app.zwipe.net/ (or /decks), open About from the header, then Decks, then Create (or load /about directly, then Decks, Create).
2. Type a deck name.
3. Click header Back: nothing happens, URL stays /decks/new, no dialog. Same for the Decks link, the logo and browser Back.

Control: load / or /decks, go to Create without visiting About, type a name, Back: the dialog shows (Stay / Leave). Format changes, picker Discard / Keep editing, Swipe piles (including empty ones), commander/partner picks and the Pir autofill don't break it; only About does.

Scotty, 2026-11-08: not only About. Home, then Decks, then + (Create), type something, Back: no prompt either. So it may be any in-app navigation before Create (or Home specifically), not About alone.

## Cause
scotland: /about and /privacy sit under OpenFrame, which draws its own Shell (and a second LeaveDialog) for a signed-in user. Leaving About mounts the authed Shell's dialog first, then About's drops and blanks the shared guard slots. Fix: a dropping dialog clears only its own slots.

Later cleanup (not scheduled, Scotty OK): have OpenFrame reuse the one authed Shell when signed in so there is never a second copy.
