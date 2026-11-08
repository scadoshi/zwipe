# 305. Home's Decks panel "+" should read "Create deck"

State: approved | Owner: laptop (queued) | PR: - | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot of web Home's Decks panel, "+" beside "All decks"): make the "+" say "Create deck" so it's obvious what it does. Scotty's explicit call for this button (overrides the icons-not-words default here). Check it fits at 390 without wrapping the "Decks 13" title; same on the app's Home if it has the button.

## Fix
Branch `nits/laptop-304`. The Decks panel's + is now a "Create deck" text button (`zwipe_ui::screens::home::NEW_DECK`) beside All decks, in the shared `components::HomeDecks`, so the app's Home has it too. The corner's reserve is the two buttons' measured 12.65rem (12.7rem), and a panel under 36rem starts the title at the left edge so it never runs under the buttons (on web that is the Decks column beside Colors, up to about 1300px, and every phone). At 390 "Decks 7" and both buttons fit on one line with 31px to spare; at 360 the title ellipsizes a little. Test: `the_copy_matches_the_app` pins the label.
