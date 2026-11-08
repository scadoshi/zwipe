# 301. Edit deck still asks to leave after a picker change was discarded

State: approved | Owner: laptop | PR: - | Build seen: 1d7053e88

## Report
Droid, Android app (debug APK of release at 1d7053e88 on prod, Pixel 9a emulator), 2026-11-08. In Edit deck, opening the Format picker, tapping Pauper, closing with ‹ and choosing Discard puts Format back, but the form still counts as changed: header Back then asks "Leave without saving? Your changes to this deck will be lost." with nothing changed. Opening Edit deck and leaving straight away shows no prompt, so the Discard is what leaves it dirty. Seen after Deck tags and Oracle tags discards too.

## Repro
1. Open any deck, tap the pencil (Edit deck).
2. Tap Format, tap Pauper, tap ‹, tap Discard. Format still reads Commander.
3. Tap ‹ in the header: "Leave without saving?" appears.

## Fix
Branch `nits/laptop-301`. Discard put back only part of what a picker's picks change, in the app's shared model and on web alike. A format pick clears the commander, and with it the partner and the background, but the Format snapshot held only the format, land target, commander and signature spell, so a deck with a partner or a Background came back without them. Discarding a Deck tags change reseeded the oracle tags from the catalog, while a loaded deck's seed comes from the compiled map; where the two differ (or the user had removed a seeded tag) the oracle tags changed. Now each picker snapshots everything its picks touch and Discard puts exactly that back: Format adds the partner and the background (card, input text and search), Deck tags keeps the oracle tags and their applied seed and no longer reseeds on Discard. Oracle tags already restored exactly. Failing tests first: zwipe-ui's `discarding_a_format_pick_leaves_edit_unchanged` and `discarding_a_deck_tag_change_leaves_edit_unchanged` (plus `discarding_an_oracle_tag_change_leaves_edit_unchanged`), and web's `discarding_a_format_pick_puts_the_whole_command_zone_back` and `discarding_a_deck_tag_change_puts_the_oracle_tags_and_seed_back`.
