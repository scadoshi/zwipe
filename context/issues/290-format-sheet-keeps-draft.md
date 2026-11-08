# 290. The Format sheet keeps its search and pick after ×

State: reproduced | Owner: laptop | PR: - | Build seen: 1e03520ef

## Report
Surfer, live web, Create/Edit deck: type 'pau', tap Pauper, close with ×, reopen: the search still says 'pau' and Pauper is still selected. Account sheets clear on close (254). Unclear whether the Keep/Discard prompt appeared and which was chosen; needs a careful repro before anything is built.

## Repro
Surfer, live, Create deck, wide: tapping Pauper fills the Format field at once. × shows "Discard format change? The form goes back to how it was when you opened this." with Keep and Discard. Keep (or a click outside) closes only the prompt; the sheet stays open with 'pau' and Pauper. Discard closes the sheet and resets Format to Not set, but reopening still shows 'pau' in the search with the list filtered.

Part 1 (inside 254's approved scope, easy): closing the sheet, by Discard or ✓, clears its search text.
Part 2 (for Scotty): Keep reads as "keep the change" but only returns to the open sheet, where ✓ still has to be tapped. Rename it (e.g. "Keep editing") or make Keep apply and close?

## Fix
Part 1, branch `nits/laptop-290`. The Format, Deck tags and Oracle tags pickers stay mounted while shut, in the app and on web, and none of them cleared its search, so a reopened picker showed the old text and a filtered list. `components::use_picker_search(open)` holds a picker's search and empties it whenever the picker closes, by ✓, Discard or a quiet close. All six pickers (three per client) use it. Failing test first: `closing_a_picker_empties_its_search`. The pick itself still follows the form: Discard puts the format back, ✓ keeps it. Part 2 (the Keep wording) waits for Scotty.

Part 2, branch `nits/laptop-290b`. The discard dialog's Keep button reads "Keep editing" (`zwipe_ui::deck::pickers::KEEP`), the one label the app and web both use for every picker that asks before discarding: Format, Deck tags and Oracle tags. No other dialog uses it. Behavior is unchanged: picks still apply only on ✓. Failing test first: `the_discard_dialog_offers_keep_editing_and_discard`.

## Decision (part 2)
Scotty, 2026-11-08: rename Keep to "Keep editing". Changes still apply only on ✓, so people can click through formats and read the descriptions without the sheet closing.
