# 442. Profile and Cards share one up-to-date deck so a star or a Warnings fix never reads a stale copy

State: fixing | Owner: laptop | PR: - (pushed branches, hashes in the composer's log) | Build seen: 2546917e1 | Type: issue

## Report
thinker, tracing "Fix to 1 clears the MVP star" and a stale Undo: the server keeps the star (thinker's zerver test), but the deck the Profile and the app's `HeldDeck` held (430, 438) was a snapshot the Cards' edits never reached: a star made in the Cards was gone when the app's Cards reopened from it with no fetch, and the Profile's Warnings fix took its before-quantity from it.

## Fix
laptop.
- App: the Cards screen writes its deck as it stands (`deck_cards::State::deck`: the profile with its main board count, the entries with their quantities, boards and stars, the command zone, the warnings over them) into `HeldDeck` on every change, and the deck view takes it (`deck_view::Action::DeckEdited`), so the Profile, its counts and its fixes read the same deck the Cards edit; the reload on close goes, since it could land before the Cards' last writes.
- Web: the phone Profile and the Cards already read the editor's one live deck; the Warnings quantity fix now goes through the editor's quantity edit, so its undo records the copies as they stand.

## Checked
- Tests (zwipe-ui): a star made in the Cards is on the deck handed back and on a screen reopened from it; the handed-back count follows an edit; a fix after an edit in the Cards records the copies the Cards left; an edit for another deck changes nothing. Web, headless: a star in the Cards survives closing and reopening them.

## Verification
-
