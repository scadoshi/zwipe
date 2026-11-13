# 472. The deck page's Buy, Clone and Share dialogs don't take focus, and Stacks logs a ResizeObserver loop

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Surfer's wide deck-page walkthrough (composer, 2026-11-13): (a) the Buy, Clone and Share dialogs don't take focus when they open, not even Clone's name field; focus their first field or primary control as the account sheets do. (b) A "ResizeObserver loop" console error on every Stacks render; find the cause and stop it without changing layout.

## Before
- (a) 1440, owner deck: focus the dock's Share, Clone or Buy and click it: the dialog opens with focus still on the dock button behind it (1 of 1 each).
- (b) 1440, owner deck, with the card images unavailable so proxies draw (102 on the page): switching to Stacks fires the error once (3 of 3); with every image loaded no proxies draw and it doesn't fire, which is why it only showed in some runs.

## Cause
- (a) The kit's `AlertDialog` takes no focus, and the web's only dialog focus (`HINT_FOCUS_JS`, `zwiper-web/src/lib/inbound/app.rs`) serves hints alone.
- (b) `components/src/deck_cards_view.rs`, `STACKS_WIDTH_JS`: the Stacks' width observer sends each width at once, inside its callback; the columns it lays out can bring the page's scrollbar and change the observed width again in the same frame, which the browser reports as a loop. A trace of every observer showed this one firing twice just before each error; deferring the card proxies' fit (the other observer then) alone left the error, deferring this one alone cleared it.

## Fix
scotland:
- `DIALOG_FOCUS_JS` (`app.rs`): as any dialog but a hint opens, focus goes to its first field, else its primary (non-danger) action, else its first link, else its Cancel; as it closes, once it has faded out, focus returns to its opener.
- `STACKS_WIDTH_JS` sends the width a frame later, and only when it changed.

## Verification
- 1440: open Share, Clone, Buy from the dock: Share focuses its primary (Create link, or Copy link once there is one), Clone its name field, Buy its first store link; Esc puts focus back on the dock button.
- 1440, Stacks with images blocked: no ResizeObserver error in 6 renders (1 of 1 per render before).
