# 513. A deck's Remove warning fix is not on Undo

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: f2ff8b894

## Report
Surfer, on 512's check (composer, 2026-11-13): on the web deck page the Remove warning fix (color identity, Counterspell) takes the card out but Undo passes it by, reaching back to an earlier Quick add, toasting Already changed and turning off, 2 of 2. Only the quantity fix logs undo. Make Remove, and clear commander if it has a fix, join Undo; check the app.

## Before
Headless, fly351's "Isshin 510" with Counterspell added off its colors (through the API, since Quick add only offers Isshin's): the warning's Remove takes the card out with "Card removed", and Cards' Undo stays off, the log holding nothing from it (1 of 1 at 1440). The app's model (`zwipe_ui::screens::deck_view`) records a quantity fix's undo and nothing for a removal.

## Cause
- Web, `zwiper-web/src/lib/inbound/screens/deck/view/mod.rs`, the warning fixes' `remove`: it deletes the card and pushes nothing on the deck's undo log, where `fix_quantity` pushes `QuantityChanged`.
- App, `zwipe-ui/src/screens/deck_view/mod.rs`, landing `CardRemoved`: it toasts and refetches with no `Effect::RecordUndo`, where landing `QuantitySet` records one.

## Fix
scotland:
- `zwipe_ui::stores::undo::UndoAction::removal(entries, card)`: a removal's undo, the row as it stands at its copies.
- The web's Remove fix takes it from the deck as the page last fetched it (512's shared copy) and pushes it once the delete lands; the app's model records it as `CardRemoved` lands. Both clients' Undo already put a `Removed` row back.
- Clear commander's fix stays off Undo, by the composer's call: Edit empties an ineligible commander before that fix can show, so surfer couldn't reach it, and no undo kind restores a command zone slot.

## Verification
- `zwipe-ui` tests `a_removal_s_undo_holds_the_row_at_its_copies` and `a_removal_fix_goes_on_the_decks_undo_log` (the app's model; red before).
- Headless, the same deck at 1440 and at 390: Remove takes Counterspell out, Undo is on, and Undo puts it back with "Re-added Counterspell".
- Surfer's after on 27d9b5d23 (1009, real clicks): Quick add Counterspell, then its warning's Remove, then Undo: Already changed, Counterspell stays out and Undo turns off, 2 of 2; with a reload between the Quick add and the Remove it works. Before, headless at 1440 from a fresh Quick add (a local zerver new enough for Quick add's sort): the same, 1 of 1. Cause: the Remove fix found the card in the deck as the page's editor last fetched it (512's shared copy), which a Quick add since the fetch doesn't reach, so it pushed nothing and Undo walked back to the add. Fix: the page shares the editor's live rows instead, its edits included; the quantity fix's baseline reads them too. After, the same sequence at 1440 and at 390 (Quick add in the phone's Cards, Remove on its Profile): Undo puts Counterspell back with "Re-added Counterspell", the add still on the log under it.
