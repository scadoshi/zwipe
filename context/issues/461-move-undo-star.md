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

## Follow-up: a refused star took the move back with it
surfer, breaking 461 (two tabs): tab 1 moves Adriana to the maybeboard, tab 2 stars a third other MVP, tab 1 presses u. Both toasts showed ('already has 3 MVPs' and 'back to deck') but she stayed on the maybeboard after reload, and the Undo entry was spent.
- Cause: Undo's move back sent `board` and `mvp: true` in one PATCH; with three MVPs already starred on the server (from the other tab, so the client's own count didn't know) the server refuses the whole write (`MvpCapReached`), the move included. The undo toast was raised before the write, so it claimed a move that never happened.
- Fix: the move back goes alone; once the server takes it, the undo's toast shows and the star follows in a write of its own (shared model `AfterMove`, web `move_card` with `StarBy::Quiet`). A refused star leaves her back on the main board, unstarred, with the server's refusal toasted. A refused move raises no success toast and puts her back where she was. With three MVPs counted locally, the star isn't sent and the cap toast shows.
- Tests: `a_starred_card_moved_off_the_main_board_loses_its_star_and_undo_puts_it_back` (now: move alone, toast on landing, then the star, then a refused star leaves her moved and unstarred), `a_refused_move_back_raises_no_success_toast`, `undo_moves_back_without_its_own_move_toast`.

## Follow-up: a failed Undo spent its entry
surfer, after 8096283c4: offline, Undo of a move fails (no false toast), but the entry is gone, so back online Undo stays disabled and the move can't be undone.
- Cause: the client pops the entry before the write (app: `view.rs` Undo, web: `apply_undo`), and nothing put it back when the write failed; the model only reverted the instant change.
- Fix: the shared model keeps each undo by the ticket of the write it waits on (`undoing`; a quantity undo moves from its debounce to its write) and records it again on that write's failure, so the app's Undo comes back; the web editor's Undo puts the entry back from every branch's failure (delete, create, quantity through `undo_retry`, move, star, printing, command zone). The web's failed undo of an add also puts the row back, and a failed printing or command zone undo shows the printing the server still holds. A star refused after a successful move back stays spent (the move landed).
- Tests: `an_undo_whose_write_fails_goes_back_on_the_log` (red without the re-record), `an_undo_that_lands_stays_spent`.


## Follow-up: an offline Undo claimed success
surfer, after 85180e115: offline, Undo of an add toasted "Removed Lightning Bolt" and Undo of a printing toasted "Sol Ring printing restored" about 70ms before "Connection error", though nothing changed. The move's toast already waited.
- Cause: the shared model (`undo_writes`) and the web editor (`apply_undo`) raised the undo's toast with the instant change, before the write, for an add, a removal, a quantity, a printing and a command zone printing. The move and the star already toasted on landing.
- Fix: the model keeps each undo's toast with its entry (`Undoing`) and raises it when the write it waits on lands, or when a quantity undo's debounce nets to no write; a failure raises none. The web editor raises it in each branch's success, a quantity undo through `undo_retry`.
- Tests: `an_undo_toasts_only_once_its_write_lands`, `an_undone_removal_toasts_only_once_its_re_add_lands`, `a_quantity_undo_toasts_only_once_its_debounced_write_lands` (all red before the fix), `a_quantity_undo_that_nets_to_no_write_still_says_so` (red with the no-write toast removed).
