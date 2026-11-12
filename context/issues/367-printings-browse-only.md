# 367. Printings offers Select where nothing would save

State: fixing | Owner: laptop | PR: #568 | Build seen: production, web

## Report
Composer for Scotty, 2026-11-11 (moved from thinker): Printings opened outside a deck (Playstyle Staples, Suggested commanders, Activity, Examples, anywhere with no deck card to save to) still shows Select, and Enter toasts "Printing saved" though nothing saves. Printings should have a browse-only mode, driven by whether a save target is passed: no Select, Enter does nothing, no toast. Deck screens keep Select. Web and app.

## Cause
- Both clients' `PrintingSheet` took a required `on_save` plus a separate `read_only` flag, and only Remove and Examples set the flag.
- Activity, Playstyle's staples and the deck page's row overlays passed a save that kept nothing:
  - Activity and the staples only re-skinned the row on screen;
  - the web deck page's row overlays passed an empty closure.

## Fix
laptop, #568:
- `on_save` is optional on both clients' `PrintingSheet` (and on the web's `CardOverlaysHost`), and the `read_only` flag is gone. Without a save the sheet only browses (the model's `Mode::ReadOnly`): no Select or Save, Enter does nothing, and leaving never toasts.
- Browse-only:
  - Activity (web and app), with Activity's display-only `PickPrinting` removed from its model;
  - Playstyle's staples;
  - Examples;
  - Remove cards;
  - the web deck page's row overlays.
- Keep their pick:
  - the deck card view and Add, where the pick saves or swaps;
  - the commander maybeboard and Playstyle's suggested commanders, rows and swipe, where the picked printing carries into Create deck.
- The suggested-commanders rule is shared: `zwipe_ui::screens::playstyle::picks_printing` holds only for a card among the suggestions. A test was written for it first.

## Verification
-
