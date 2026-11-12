# 365. Examples keys, d on every swipe screen, ? to the open modal, the card clear of its details

State: fixing | Owner: scotland | PR: #PR | Build seen: live, web

## Report
Scotty via composer, 2026-11-12, web:
- (a) The Oracle tag Examples modal gets keys: `h` the next card (its Next, the left swipe), `j` back one (its Back one, down), `d` the card's details, `v` flip or meld, Escape closes.
- (b) `d` opens or closes the card's details on every swipe screen (Add, Remove, Commander swipe, Suggested commanders swipe, Examples), wired through the `card_key` map and listed in the Shift+H sheet.
- (c) When the docked details would overlap the card and there is room on the left to fit the card clear of them, the card slides left just enough, eased like the panel. Without the room it stays and the panel overlaps it; a panel that misses the card moves nothing. It slides back when the details close. This lives in the shared swipe and details layout.
- (d) With the Examples modal open over Playstyle, `?` opened Playstyle's hint. `?` must open the hint of the topmost open modal or sheet that has one, falling back to the screen's only when no modal is open, solved once in the shared key routing.

## Cause
- (a) and (b): the Examples modal listened only for its swipes (`h` and `j` already swiped) and `v`. The swipe screens had `d`, but outside the shared map.
- (c): Add and Commander swipe shifted the card by a CSS estimate of its width, with no check for room on the left. Examples did not shift at all.
- (d): the global key listener sent `?` to the newest page registration. A modal that did not register (Examples, the filter sheet) never got it, and the overlay guard dropped `?` outright while a sheet was open.

## Fix
scotland, #PR (zwiper-web):
- `shortcuts::CardKey` gains `Details` (`d`). The swipe screens' map and the deck page's hover keys take `d` from it. The Examples modal answers `d` with its eye.
- `ShortcutsScreen::Examples` lists ← h Next, ↓ j Back one, d Card details and v Flip card. The modal offers the sheet while open.
- `chrome/details_clear.rs` replaces the CSS estimate with a shared script installed once at the app root. It runs on resize and on class changes, for Add, Remove, Commander swipe and the Examples:
  - It measures the top card's image against the shown panel's resting edge.
  - If they overlap and the room left of the card in its body covers the overlap, it slides the stack by exactly that much, with a 1rem gap. Otherwise it leaves the card in place.
  - It uses the stack's `translate`, with web.css easing it like the panel (0.3s ease).
- The global key script routes `?`. With an open modal or sheet (`shortcuts::MODAL_SELECTORS`: a dialog, a sheet, a large modal, Commander swipe), it presses the topmost one's own header `?`, or does nothing if that one has none. Only with no modal open does it go to the page's hint. `?` passes the overlay guard; typing and Ctrl, Alt or Meta still block it.

Tests: `card_key` pins `d` on the swipe screens; every key the Examples sheet lists works there; the `?` branch goes to the modal before the page; the slide covers every surface and moves only when it overlaps and fits.

Headless on scotland, Examples opened from Playstyle:
- `d` toggles the details; `h` and `j` step forward and back.
- `?` opens "Example cards" (the modal's hint), not Playstyle's.
- Shift+H lists the modal's keys.
- The slide: at 1600 the card moves 10px (it overlapped by that much); at 1300 it stays (the panel misses it); at 1100 and 950 it moves 76px and 151px and clears; on close it returns.
- Add at 1100 moves 43px; at 1600 it stays.

Shots `target/shots-365/365-{1600,1300,1100,950}.png`.
