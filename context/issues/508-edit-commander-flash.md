# 508. Edit deck flashes "Not set" in Commander before the saved commander fills in

State: fixing | Owner: laptop | Type: issue | PR: - (pushed branch, hash in the composer's log) | Build seen: release 869fced20

## Report
droider, via the composer: the app's Edit deck form shows "Not set" in the Commander field for about 2 frames right after the skeleton, before "Isshin, Two Heavens as One" fills in (halo `611/after/compare-493-app-format.mp4`, about 0.7 s). The form should wait for the commander, or the field should stay a ghost. Check the web too.

## Before
- App: droider's clip above.
- Web, local (laptop297's Deck 1, Commander Atraxa), with the `/api/card/<id>` fetch held by CDP: the skeleton is gone and the form shows with the Commander field empty under its "Not set" placeholder; on release, Atraxa fills it.

## Cause
Both forms draw as soon as the deck's profile arrives, then fetch each saved command-zone card (commander, partner, background, signature spell) by id on its own. Until that fetch answers, the slot is empty.
- App: `zwipe_ui::screens::deck_form::State::loaded` set `Status::Ready` with the deck, before its `Effect::FetchCard`s answered.
- Web: `screens/deck/edit.rs` swapped the skeleton for `DeckFields` once the deck resource resolved, while the card fetches it spawns were still out.

## Fix
- App: Edit stays `Status::Loading` (the skeleton) until no command-zone card fetch waits; a card that fails to load still lets the form draw, with that slot empty.
- Web: `edit.rs` counts the card fetches still out and draws the form once every one has answered, either way.
- A deck with no command zone draws as it loads, as before.

## Checked
- `zwipe-ui` tests, red before: `edit_loads_the_deck_then_its_command_zone` (Loading after the deck and after the partner, Ready after the commander) and `a_failed_card_load_leaves_the_slot_empty` (Loading until the failure); negative control `a_deck_with_no_command_zone_draws_as_it_loads`.
- Web after, same setup, 3 of 3: with the card held, the skeleton shows (no fields); on release, the form draws with Atraxa already in Commander. Deck 3 (no commander) draws at once.

## Not checked
- The app on a device or simulator: it draws `status()` from the same model the tests cover.

## Verification
-
