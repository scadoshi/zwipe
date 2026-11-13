# 461. A starred card moved to the maybeboard and undone comes back unstarred

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: 4059d64f8

## Report
droider, breaking 449 (phone web, live 4059d64f8, deck Surfer 399 Isshin, Adriana x4 the only MVP): Cards, Stacks, open Adriana's details, To maybeboard, Undo, reload. Expected: Adriana x4 on the main board and still starred. Actual: back with 4 copies but not starred on the server; the open web page kept showing her as an MVP until reload, the app showed her unstarred. 1 of 1.

## Cause
- Server, `zerver/src/lib/outbound/sqlx/deck/mod.rs` `update_deck_card`: a move off the main board sets `mvp_at = NULL` (MVPs are main board only). Every board move does this, To sideboard included.
- Clients: the undo entry `UndoAction::MovedBoard` kept only the board the card left, and Undo's move back sent `board` alone, so the server put the card back unstarred.
- The web showed the star until reload because its move never cleared `mvp_at` locally; the app's shared model also kept it locally but the app refetches the deck through its held deck.

## Fix
thinker:
- `MovedBoard` carries `starred`: the star time the move took off the main board. The move clears the star locally to match the server and takes the server's star from the reply (shared deck cards model, web editor `move_card`); a refused move puts board and star back.
- Undo's move back to the main board stars it again in the same write (`board` and `mvp: true` in one PATCH), so the server's main-board rule holds; with three other MVPs starred since, it moves back alone, since the star would be refused and take the move with it.
- Server test `a_move_back_with_its_star_restores_a_star_the_move_dropped` pins the server side: the move drops the star, a move back alone leaves it off, a move back with `mvp: true` restores it.

## Not covered
- The Remove screen's swipe to the maybeboard drops the star the same way, and its own gesture undo moves back without it; its entries record `starred: None`.
- A restored star gets a new star time from the server, so the card can move along the featured strip.

## Verification
- `cargo test -p zwipe-ui` (`a_starred_card_moved_off_the_main_board_loses_its_star_and_undo_puts_it_back`, red with the restar removed) and `cargo test -p zerver --test deck_cards a_move_back`.
- droider's steps on web and app: Adriana is starred after Undo and after a reload.
