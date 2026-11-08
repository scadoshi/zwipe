# 290. The Format sheet keeps its search and pick after ×

State: reproduced | Owner: laptop (part 1) | PR: - | Build seen: 1e03520ef

## Report
Surfer, live web, Create/Edit deck: type 'pau', tap Pauper, close with ×, reopen: the search still says 'pau' and Pauper is still selected. Account sheets clear on close (254). Unclear whether the Keep/Discard prompt appeared and which was chosen; needs a careful repro before anything is built.

## Repro
Surfer, live, Create deck, wide: tapping Pauper fills the Format field at once. × shows "Discard format change? The form goes back to how it was when you opened this." with Keep and Discard. Keep (or a click outside) closes only the prompt; the sheet stays open with 'pau' and Pauper. Discard closes the sheet and resets Format to Not set, but reopening still shows 'pau' in the search with the list filtered.

Part 1 (inside 254's approved scope, easy): closing the sheet, by Discard or ✓, clears its search text.
Part 2 (for Scotty): Keep reads as "keep the change" but only returns to the open sheet, where ✓ still has to be tapped. Rename it (e.g. "Keep editing") or make Keep apply and close?
