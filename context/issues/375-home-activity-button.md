# 375. Home's counters have no way into Activity on the web

State: fixing | Owner: laptop | PR: - | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12: app.zwipe.net Home's counters panel (Decks created, Searches run, Cards swiped) has no way into Activity, while Decks has Create deck and All decks and Colors has Playstyle. It needs an Activity button the way the app's Home counters have one.

## Fix
laptop: the web's `HomeHeroScoped` takes `onactivity` and ends its chips row with an Activity `card-action-btn`, after Live (or "as of") and the scope button. That is where and how the app's hero has it, and the same button the Decks and Colors headers use. Home sends the shared model's `Action::Activity`, which already routes to `/activity`.

## Verification
-
