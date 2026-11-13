# 467. The Remove screen's swipe to the maybeboard drops a card's star, and its undo moves it back unstarred

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: c439fd91a

## Report
thinker's finding while fixing 461: on the Remove screen, the swipe up that moves a starred main-board card to the maybeboard drops its star, and the screen's own gesture undo moves it back without it; its entry on the deck's undo log recorded no star either.

## Cause
- The server drops the star on any move off the main board (`update_deck_card`, `mvp_at = NULL`).
- The Remove screen's move kept no star: `RemoveAction::MoveBoard` held only the boards (`zwipe-ui/src/stack/mod.rs`), the deck's undo entry was recorded with `starred: None` (`screens/remove_cards`, the web's `cards/remove.rs`), and the gesture undo sent the board alone (`RestoreBoard`), so the card came back unstarred.

## Fix
thinker:
- The move takes the star time it drops onto `RemoveAction::MoveBoard` and the deck's undo entry (`MovedBoard.starred`), and clears the star locally to match the server.
- The gesture undo's move back goes alone; once it lands, a new `RestoreStar` (`with_mvp(true)`) puts the star back in a write of its own, so a refused star leaves the card back on the main board, unstarred, with the server's word toasted. With three MVPs counted, no star is sent and the cap toast shows. App through the shared model, web in its own `remove.rs`.

## Verification
- `cargo test -p zwipe-ui remove_cards` (`a_starred_card_swiped_to_the_maybeboard_gets_its_star_back_on_undo`, red with the restar removed; `a_refused_star_leaves_the_undone_move_standing_unstarred`).
- Remove screen, web and app: star a card, swipe it up (to the maybeboard), undo: it's back on the main board and starred, also after reload. Then from the deck's Cards, Undo of the same move (if the screen's own undo wasn't used) puts it back starred too.
