# 352. Back loops between Create deck and Decks

State: fixing | Owner: laptop | PR: - | Build seen: live (recording `target/352-back-loop.mov`)

## Report
Scotty, 2026-11-11, web: on Playstyle, Suggested commanders, a swipe right opens Create deck. The header's Back then goes to Decks instead of back to Playstyle. Back on Decks returns to Create deck, and Back again to Decks: a loop. Surfer also saw that moving between two deck pages could leave the URL and title on one deck and the page on the other until a reload.

## Cause
- The header's Back pushed a screen's fixed parent instead of going back. Create deck pushed Decks; Back on Decks went back to Create deck, which pushed Decks again. The deck page pushed its opener and the deck screens pushed the deck the same way.
- On the web, `can_go_back` is always true (Dioxus's default for the browser history), so Back could not tell a deep link from a screen with the app behind it.
- Saving a new deck pushed the deck over the spent form, so Back from the new deck returned to Create deck. Saving an edit pushed the deck over Edit.
- The app has the same pushes: Edit deck's Back and save, Import and Export's Back, the deck page's Back (its opener), the maybeboard's Back (Decks) and Decks' Back (Home). The OS back (`go_back`) then returned through them.
- The web deck page kept its state when the route moved from one deck to another (a clone, or the browser's back and forward between two decks): the profile fetched once on mount, so the heading kept the first deck's name while the URL and title showed the second.

## Repro
1. Web, logged in: Playstyle, Suggested commanders, swipe right (or Decks, Create).
2. Header Back: Decks, not the screen before Create deck.
3. Back again: Create deck again, and so on.
4. A deck, Clone, Save: the clone's URL and title, the original's name in the heading. Browser back and forward keep the stale heading.

## Fix
laptop, `issue/352-back-loop`:
- Web: `chrome/trail.rs` keeps the tab's own trail of app screens over the router's history (above the leave guard), so `can_go_back` is true only with an app screen behind. Every header Back goes back when there is one and otherwise replaces this screen with its parent (`nav::back_or`); Back never pushes.
- Web: Create's save replaces the form with the new deck; Edit's save goes back to the deck (or replaces Edit with it on a deep link).
- Web: the deck page mounts afresh per deck (`ViewDeck` keys its body by the deck id), so a clone or the browser's back and forward between two decks shows the right deck.
- App: `back_handler::back_or` for every Back that named a fixed screen (the deck page, Edit, Import, Export, the maybeboard, Decks). The deck form's model replaces Create with the new deck (`Effect::Replace`).
- Tests first: `chrome::trail` (fresh tab, push, replace, back and forward, a pop the guard undid, a longer jump) and `create_sends_the_form_and_opens_the_new_deck` (zwipe-ui, now a replace).

## Verification
-
